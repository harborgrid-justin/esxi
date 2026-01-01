/**
 * Enterprise API Gateway - Distributed Rate Limiter
 *
 * Redis-backed distributed rate limiting for multi-instance deployments
 */

import Redis from 'ioredis';

export interface DistributedLimitResult {
  allowed: boolean;
  remaining: number;
  resetAt: number;
  retryAfter?: number;
}

export class DistributedLimiter {
  private redis: Redis;

  constructor(redisConfig: { host: string; port: number; password?: string; db?: number }) {
    this.redis = new Redis({
      host: redisConfig.host,
      port: redisConfig.port,
      password: redisConfig.password,
      db: redisConfig.db || 0,
      retryStrategy: (times: number) => {
        const delay = Math.min(times * 50, 2000);
        return delay;
      },
    });
  }

  /**
   * Token bucket implementation using Redis
   */
  public async tokenBucket(
    key: string,
    limit: number,
    refillRate: number,
    burstSize: number = limit
  ): Promise<DistributedLimitResult> {
    const now = Date.now();
    const script = `
      local key = KEYS[1]
      local limit = tonumber(ARGV[1])
      local refill_rate = tonumber(ARGV[2])
      local burst_size = tonumber(ARGV[3])
      local now = tonumber(ARGV[4])

      local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
      local tokens = tonumber(bucket[1]) or burst_size
      local last_refill = tonumber(bucket[2]) or now

      -- Refill tokens
      local elapsed = (now - last_refill) / 1000
      local tokens_to_add = elapsed * refill_rate
      tokens = math.min(burst_size, tokens + tokens_to_add)

      -- Check if allowed
      local allowed = tokens >= 1
      local remaining = tokens

      if allowed then
        tokens = tokens - 1
        remaining = tokens
      end

      -- Update state
      redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
      redis.call('EXPIRE', key, 3600)

      -- Calculate reset time
      local tokens_needed = limit - tokens
      local reset_at = now + (tokens_needed / refill_rate * 1000)

      return {allowed and 1 or 0, math.floor(remaining), math.floor(reset_at)}
    `;

    const result = (await this.redis.eval(
      script,
      1,
      key,
      limit.toString(),
      refillRate.toString(),
      burstSize.toString(),
      now.toString()
    )) as [number, number, number];

    const allowed = result[0] === 1;
    const remaining = result[1] || 0;
    const resetAt = result[2] || now;

    return {
      allowed,
      remaining,
      resetAt,
      retryAfter: allowed ? undefined : resetAt - now,
    };
  }

  /**
   * Sliding window implementation using Redis
   */
  public async slidingWindow(
    key: string,
    limit: number,
    windowMs: number
  ): Promise<DistributedLimitResult> {
    const now = Date.now();
    const windowStart = now - windowMs;

    const script = `
      local key = KEYS[1]
      local limit = tonumber(ARGV[1])
      local window_start = tonumber(ARGV[2])
      local now = tonumber(ARGV[3])
      local window_ms = tonumber(ARGV[4])

      -- Remove old entries
      redis.call('ZREMRANGEBYSCORE', key, 0, window_start)

      -- Count current entries
      local count = redis.call('ZCARD', key)

      local allowed = count < limit
      local remaining = math.max(0, limit - count)

      if allowed then
        redis.call('ZADD', key, now, now)
        remaining = remaining - 1
      end

      redis.call('EXPIRE', key, math.ceil(window_ms / 1000) + 1)

      -- Get oldest entry for retry calculation
      local oldest = redis.call('ZRANGE', key, 0, 0, 'WITHSCORES')
      local retry_after = 0
      if #oldest > 0 then
        retry_after = tonumber(oldest[2]) + window_ms - now
      end

      return {allowed and 1 or 0, remaining, now + window_ms, retry_after}
    `;

    const result = (await this.redis.eval(
      script,
      1,
      key,
      limit.toString(),
      windowStart.toString(),
      now.toString(),
      windowMs.toString()
    )) as [number, number, number, number];

    const allowed = result[0] === 1;
    const remaining = result[1] || 0;
    const resetAt = result[2] || now + windowMs;
    const retryAfter = result[3] || 0;

    return {
      allowed,
      remaining,
      resetAt,
      retryAfter: allowed ? undefined : Math.max(0, retryAfter),
    };
  }

  /**
   * Fixed window implementation using Redis
   */
  public async fixedWindow(
    key: string,
    limit: number,
    windowMs: number
  ): Promise<DistributedLimitResult> {
    const now = Date.now();
    const windowStart = Math.floor(now / windowMs) * windowMs;
    const windowKey = `${key}:${windowStart}`;

    const script = `
      local key = KEYS[1]
      local limit = tonumber(ARGV[1])
      local window_ms = tonumber(ARGV[2])
      local reset_at = tonumber(ARGV[3])

      local count = tonumber(redis.call('GET', key) or 0)
      local allowed = count < limit
      local remaining = math.max(0, limit - count)

      if allowed then
        redis.call('INCR', key)
        redis.call('PEXPIRE', key, window_ms)
        remaining = remaining - 1
      end

      return {allowed and 1 or 0, remaining, reset_at}
    `;

    const resetAt = windowStart + windowMs;
    const result = (await this.redis.eval(
      script,
      1,
      windowKey,
      limit.toString(),
      windowMs.toString(),
      resetAt.toString()
    )) as [number, number, number];

    const allowed = result[0] === 1;
    const remaining = result[1] || 0;

    return {
      allowed,
      remaining,
      resetAt,
      retryAfter: allowed ? undefined : resetAt - now,
    };
  }

  /**
   * Reset rate limit for a key
   */
  public async reset(key: string): Promise<void> {
    await this.redis.del(key);
  }

  /**
   * Get current count for a key (sliding window)
   */
  public async getCount(key: string, windowMs: number): Promise<number> {
    const now = Date.now();
    const windowStart = now - windowMs;

    await this.redis.zremrangebyscore(key, 0, windowStart);
    const count = await this.redis.zcard(key);

    return count;
  }

  /**
   * Close Redis connection
   */
  public async close(): Promise<void> {
    await this.redis.quit();
  }

  /**
   * Get Redis info
   */
  public async getInfo(): Promise<string> {
    return this.redis.info();
  }
}
