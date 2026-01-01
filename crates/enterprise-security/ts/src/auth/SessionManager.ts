/**
 * Session Manager - Secure Session Management
 * Enterprise session management with encryption and multi-device support
 */

import { nanoid } from 'nanoid';
import * as crypto from 'crypto';

// ============================================================================
// Types
// ============================================================================

export interface Session {
  id: string;
  userId: string;
  deviceId: string;
  ipAddress: string;
  userAgent: string;
  createdAt: Date;
  expiresAt: Date;
  lastActivityAt: Date;
  refreshToken?: string;
  metadata: SessionMetadata;
}

export interface SessionMetadata {
  location?: {
    country?: string;
    region?: string;
    city?: string;
  };
  device?: {
    type: 'desktop' | 'mobile' | 'tablet' | 'unknown';
    os?: string;
    browser?: string;
  };
  mfaVerified: boolean;
  securityLevel: 'low' | 'medium' | 'high';
}

export interface SessionConfig {
  sessionDuration: number; // milliseconds
  refreshTokenDuration: number;
  maxSessionsPerUser: number;
  requireMFAForSensitive: boolean;
  idleTimeout: number;
}

export interface DeviceInfo {
  id: string;
  name: string;
  type: string;
  lastUsed: Date;
  trusted: boolean;
}

// ============================================================================
// Session Manager Implementation
// ============================================================================

export class SessionManager {
  private sessions: Map<string, Session> = new Map();
  private userSessions: Map<string, Set<string>> = new Map();
  private config: SessionConfig = {
    sessionDuration: 24 * 60 * 60 * 1000, // 24 hours
    refreshTokenDuration: 7 * 24 * 60 * 60 * 1000, // 7 days
    maxSessionsPerUser: 5,
    requireMFAForSensitive: true,
    idleTimeout: 30 * 60 * 1000, // 30 minutes
  };

  /**
   * Create new session
   */
  async createSession(
    userId: string,
    deviceId: string,
    ipAddress: string,
    userAgent: string,
    mfaVerified: boolean = false
  ): Promise<Session> {
    // Enforce max sessions per user
    await this.enforceSesssionLimit(userId);

    const now = new Date();
    const session: Session = {
      id: this.generateSessionId(),
      userId,
      deviceId,
      ipAddress,
      userAgent,
      createdAt: now,
      expiresAt: new Date(now.getTime() + this.config.sessionDuration),
      lastActivityAt: now,
      refreshToken: this.generateRefreshToken(),
      metadata: {
        device: this.parseUserAgent(userAgent),
        mfaVerified,
        securityLevel: this.calculateSecurityLevel(mfaVerified, deviceId),
      },
    };

    this.sessions.set(session.id, session);

    // Track user sessions
    const userSessionIds = this.userSessions.get(userId) || new Set();
    userSessionIds.add(session.id);
    this.userSessions.set(userId, userSessionIds);

    return session;
  }

  /**
   * Get session by ID
   */
  async getSession(sessionId: string): Promise<Session | null> {
    const session = this.sessions.get(sessionId);
    if (!session) {
      return null;
    }

    // Check expiration
    if (this.isExpired(session)) {
      await this.terminateSession(sessionId);
      return null;
    }

    // Check idle timeout
    if (this.isIdle(session)) {
      await this.terminateSession(sessionId);
      return null;
    }

    return session;
  }

  /**
   * Validate and refresh session
   */
  async validateSession(sessionId: string): Promise<boolean> {
    const session = await this.getSession(sessionId);
    if (!session) {
      return false;
    }

    // Update last activity
    session.lastActivityAt = new Date();
    return true;
  }

  /**
   * Refresh session with refresh token
   */
  async refreshSession(sessionId: string, refreshToken: string): Promise<Session | null> {
    const session = this.sessions.get(sessionId);
    if (!session || session.refreshToken !== refreshToken) {
      return null;
    }

    // Create new session
    const newSession = await this.createSession(
      session.userId,
      session.deviceId,
      session.ipAddress,
      session.userAgent,
      session.metadata.mfaVerified
    );

    // Terminate old session
    await this.terminateSession(sessionId);

    return newSession;
  }

  /**
   * Update session metadata
   */
  async updateSessionMetadata(
    sessionId: string,
    metadata: Partial<SessionMetadata>
  ): Promise<void> {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new Error('Session not found');
    }

    session.metadata = {
      ...session.metadata,
      ...metadata,
    };
  }

  /**
   * Terminate session
   */
  async terminateSession(sessionId: string): Promise<void> {
    const session = this.sessions.get(sessionId);
    if (!session) {
      return;
    }

    // Remove from maps
    this.sessions.delete(sessionId);

    const userSessionIds = this.userSessions.get(session.userId);
    if (userSessionIds) {
      userSessionIds.delete(sessionId);
      if (userSessionIds.size === 0) {
        this.userSessions.delete(session.userId);
      }
    }
  }

  /**
   * Terminate all user sessions
   */
  async terminateAllUserSessions(userId: string): Promise<void> {
    const sessionIds = this.userSessions.get(userId);
    if (!sessionIds) {
      return;
    }

    for (const sessionId of Array.from(sessionIds)) {
      await this.terminateSession(sessionId);
    }
  }

  /**
   * Terminate all user sessions except current
   */
  async terminateOtherSessions(userId: string, currentSessionId: string): Promise<void> {
    const sessionIds = this.userSessions.get(userId);
    if (!sessionIds) {
      return;
    }

    for (const sessionId of Array.from(sessionIds)) {
      if (sessionId !== currentSessionId) {
        await this.terminateSession(sessionId);
      }
    }
  }

  /**
   * Get all user sessions
   */
  getUserSessions(userId: string): Session[] {
    const sessionIds = this.userSessions.get(userId);
    if (!sessionIds) {
      return [];
    }

    const sessions: Session[] = [];
    for (const sessionId of sessionIds) {
      const session = this.sessions.get(sessionId);
      if (session && !this.isExpired(session)) {
        sessions.push(session);
      }
    }

    return sessions;
  }

  /**
   * Get user devices
   */
  getUserDevices(userId: string): DeviceInfo[] {
    const sessions = this.getUserSessions(userId);
    const devicesMap = new Map<string, DeviceInfo>();

    for (const session of sessions) {
      if (!devicesMap.has(session.deviceId)) {
        devicesMap.set(session.deviceId, {
          id: session.deviceId,
          name: this.getDeviceName(session.metadata.device),
          type: session.metadata.device?.type || 'unknown',
          lastUsed: session.lastActivityAt,
          trusted: session.metadata.securityLevel === 'high',
        });
      } else {
        const device = devicesMap.get(session.deviceId)!;
        if (session.lastActivityAt > device.lastUsed) {
          device.lastUsed = session.lastActivityAt;
        }
      }
    }

    return Array.from(devicesMap.values());
  }

  /**
   * Revoke device (terminate all sessions)
   */
  async revokeDevice(userId: string, deviceId: string): Promise<void> {
    const sessions = this.getUserSessions(userId);
    for (const session of sessions) {
      if (session.deviceId === deviceId) {
        await this.terminateSession(session.id);
      }
    }
  }

  /**
   * Clean up expired sessions
   */
  async cleanupExpiredSessions(): Promise<number> {
    let cleaned = 0;
    const now = new Date();

    for (const [sessionId, session] of this.sessions.entries()) {
      if (session.expiresAt < now) {
        await this.terminateSession(sessionId);
        cleaned++;
      }
    }

    return cleaned;
  }

  /**
   * Get session statistics
   */
  getSessionStats(): {
    totalSessions: number;
    activeUsers: number;
    averageSessionsPerUser: number;
  } {
    return {
      totalSessions: this.sessions.size,
      activeUsers: this.userSessions.size,
      averageSessionsPerUser:
        this.userSessions.size > 0 ? this.sessions.size / this.userSessions.size : 0,
    };
  }

  /**
   * Configure session settings
   */
  configure(config: Partial<SessionConfig>): void {
    this.config = {
      ...this.config,
      ...config,
    };
  }

  // ============================================================================
  // Private Helper Methods
  // ============================================================================

  private generateSessionId(): string {
    return nanoid(32);
  }

  private generateRefreshToken(): string {
    return crypto.randomBytes(32).toString('base64url');
  }

  private isExpired(session: Session): boolean {
    return new Date() > session.expiresAt;
  }

  private isIdle(session: Session): boolean {
    const idleTime = Date.now() - session.lastActivityAt.getTime();
    return idleTime > this.config.idleTimeout;
  }

  private async enforceSesssionLimit(userId: string): Promise<void> {
    const sessions = this.getUserSessions(userId);
    if (sessions.length >= this.config.maxSessionsPerUser) {
      // Remove oldest session
      const oldest = sessions.sort((a, b) =>
        a.lastActivityAt.getTime() - b.lastActivityAt.getTime()
      )[0];

      if (oldest) {
        await this.terminateSession(oldest.id);
      }
    }
  }

  private calculateSecurityLevel(
    mfaVerified: boolean,
    deviceId: string
  ): 'low' | 'medium' | 'high' {
    if (mfaVerified) {
      return 'high';
    }
    // Could check if device is trusted
    return 'medium';
  }

  private parseUserAgent(userAgent: string): SessionMetadata['device'] {
    // Simplified user agent parsing
    let type: 'desktop' | 'mobile' | 'tablet' | 'unknown' = 'unknown';

    if (/mobile/i.test(userAgent)) {
      type = 'mobile';
    } else if (/tablet|ipad/i.test(userAgent)) {
      type = 'tablet';
    } else if (/mozilla/i.test(userAgent)) {
      type = 'desktop';
    }

    let os: string | undefined;
    if (/windows/i.test(userAgent)) os = 'Windows';
    else if (/mac os/i.test(userAgent)) os = 'macOS';
    else if (/linux/i.test(userAgent)) os = 'Linux';
    else if (/android/i.test(userAgent)) os = 'Android';
    else if (/ios|iphone|ipad/i.test(userAgent)) os = 'iOS';

    let browser: string | undefined;
    if (/chrome/i.test(userAgent)) browser = 'Chrome';
    else if (/safari/i.test(userAgent)) browser = 'Safari';
    else if (/firefox/i.test(userAgent)) browser = 'Firefox';
    else if (/edge/i.test(userAgent)) browser = 'Edge';

    return { type, os, browser };
  }

  private getDeviceName(device?: SessionMetadata['device']): string {
    if (!device) return 'Unknown Device';

    const parts: string[] = [];
    if (device.browser) parts.push(device.browser);
    if (device.os) parts.push(device.os);
    if (device.type) parts.push(device.type);

    return parts.length > 0 ? parts.join(' - ') : 'Unknown Device';
  }
}

// Export singleton instance
export const sessionManager = new SessionManager();
