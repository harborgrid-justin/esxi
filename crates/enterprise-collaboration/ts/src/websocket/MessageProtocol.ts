/**
 * Binary Message Protocol
 * Efficient binary encoding/decoding for WebSocket messages
 */

import { Message, MessageType } from '../types';

export enum ProtocolVersion {
  V1 = 1,
}

/**
 * Binary Message Format:
 * [Version: 1 byte]
 * [Message Type: 1 byte]
 * [Timestamp: 8 bytes]
 * [Sender ID Length: 2 bytes]
 * [Sender ID: variable]
 * [Message ID Length: 2 bytes]
 * [Message ID: variable]
 * [Payload Length: 4 bytes]
 * [Payload: variable (JSON)]
 */
export class MessageProtocol {
  private encoder = new TextEncoder();
  private decoder = new TextDecoder();

  /**
   * Encode a message to binary format
   */
  encode<T = unknown>(message: Message<T>): ArrayBuffer {
    const senderIdBytes = this.encoder.encode(message.senderId);
    const messageIdBytes = this.encoder.encode(message.id);
    const payloadBytes = this.encoder.encode(JSON.stringify(message.payload));

    const totalSize =
      1 + // version
      1 + // message type
      8 + // timestamp
      2 + senderIdBytes.length + // sender ID
      2 + messageIdBytes.length + // message ID
      4 + payloadBytes.length; // payload

    const buffer = new ArrayBuffer(totalSize);
    const view = new DataView(buffer);
    let offset = 0;

    // Version
    view.setUint8(offset, ProtocolVersion.V1);
    offset += 1;

    // Message Type
    view.setUint8(offset, this.messageTypeToNumber(message.type));
    offset += 1;

    // Timestamp
    view.setBigInt64(offset, BigInt(message.timestamp.getTime()));
    offset += 8;

    // Sender ID
    view.setUint16(offset, senderIdBytes.length);
    offset += 2;
    new Uint8Array(buffer, offset, senderIdBytes.length).set(senderIdBytes);
    offset += senderIdBytes.length;

    // Message ID
    view.setUint16(offset, messageIdBytes.length);
    offset += 2;
    new Uint8Array(buffer, offset, messageIdBytes.length).set(messageIdBytes);
    offset += messageIdBytes.length;

    // Payload
    view.setUint32(offset, payloadBytes.length);
    offset += 4;
    new Uint8Array(buffer, offset, payloadBytes.length).set(payloadBytes);

    return buffer;
  }

  /**
   * Decode a binary message
   */
  decode<T = unknown>(data: ArrayBuffer | Blob | string): Message<T> {
    // Handle different input types
    if (data instanceof Blob) {
      throw new Error('Blob decoding not supported synchronously');
    }

    if (typeof data === 'string') {
      // Fallback to JSON parsing for text messages
      return JSON.parse(data);
    }

    const view = new DataView(data);
    let offset = 0;

    // Version
    const version = view.getUint8(offset);
    offset += 1;

    if (version !== ProtocolVersion.V1) {
      throw new Error(`Unsupported protocol version: ${version}`);
    }

    // Message Type
    const messageTypeNum = view.getUint8(offset);
    const messageType = this.numberToMessageType(messageTypeNum);
    offset += 1;

    // Timestamp
    const timestamp = new Date(Number(view.getBigInt64(offset)));
    offset += 8;

    // Sender ID
    const senderIdLength = view.getUint16(offset);
    offset += 2;
    const senderIdBytes = new Uint8Array(data, offset, senderIdLength);
    const senderId = this.decoder.decode(senderIdBytes);
    offset += senderIdLength;

    // Message ID
    const messageIdLength = view.getUint16(offset);
    offset += 2;
    const messageIdBytes = new Uint8Array(data, offset, messageIdLength);
    const messageId = this.decoder.decode(messageIdBytes);
    offset += messageIdLength;

    // Payload
    const payloadLength = view.getUint32(offset);
    offset += 4;
    const payloadBytes = new Uint8Array(data, offset, payloadLength);
    const payloadStr = this.decoder.decode(payloadBytes);
    const payload = JSON.parse(payloadStr) as T;

    return {
      id: messageId,
      type: messageType,
      payload,
      senderId,
      timestamp,
    };
  }

  /**
   * Encode a message to JSON (fallback)
   */
  encodeJSON<T = unknown>(message: Message<T>): string {
    return JSON.stringify(message);
  }

  /**
   * Decode a JSON message (fallback)
   */
  decodeJSON<T = unknown>(json: string): Message<T> {
    return JSON.parse(json);
  }

  /**
   * Convert MessageType to number
   */
  private messageTypeToNumber(type: MessageType): number {
    const typeMap: Record<MessageType, number> = {
      [MessageType.CONNECT]: 0,
      [MessageType.DISCONNECT]: 1,
      [MessageType.HEARTBEAT]: 2,
      [MessageType.OPERATION]: 10,
      [MessageType.SYNC]: 11,
      [MessageType.CHECKPOINT]: 12,
      [MessageType.PRESENCE_UPDATE]: 20,
      [MessageType.CURSOR_MOVE]: 21,
      [MessageType.SELECTION_CHANGE]: 22,
      [MessageType.COMMENT_ADD]: 30,
      [MessageType.COMMENT_UPDATE]: 31,
      [MessageType.COMMENT_DELETE]: 32,
      [MessageType.COMMENT_RESOLVE]: 33,
      [MessageType.CONFLICT_DETECTED]: 40,
      [MessageType.CONFLICT_RESOLVED]: 41,
      [MessageType.ERROR]: 100,
      [MessageType.ACK]: 101,
      [MessageType.NACK]: 102,
    };

    return typeMap[type] ?? 255;
  }

  /**
   * Convert number to MessageType
   */
  private numberToMessageType(num: number): MessageType {
    const reverseMap: Record<number, MessageType> = {
      0: MessageType.CONNECT,
      1: MessageType.DISCONNECT,
      2: MessageType.HEARTBEAT,
      10: MessageType.OPERATION,
      11: MessageType.SYNC,
      12: MessageType.CHECKPOINT,
      20: MessageType.PRESENCE_UPDATE,
      21: MessageType.CURSOR_MOVE,
      22: MessageType.SELECTION_CHANGE,
      30: MessageType.COMMENT_ADD,
      31: MessageType.COMMENT_UPDATE,
      32: MessageType.COMMENT_DELETE,
      33: MessageType.COMMENT_RESOLVE,
      40: MessageType.CONFLICT_DETECTED,
      41: MessageType.CONFLICT_RESOLVED,
      100: MessageType.ERROR,
      101: MessageType.ACK,
      102: MessageType.NACK,
    };

    const type = reverseMap[num];
    if (!type) {
      throw new Error(`Unknown message type number: ${num}`);
    }

    return type;
  }

  /**
   * Calculate message size
   */
  calculateSize(message: Message): number {
    const senderIdSize = this.encoder.encode(message.senderId).length;
    const messageIdSize = this.encoder.encode(message.id).length;
    const payloadSize = this.encoder.encode(
      JSON.stringify(message.payload)
    ).length;

    return (
      1 + // version
      1 + // message type
      8 + // timestamp
      2 + senderIdSize + // sender ID
      2 + messageIdSize + // message ID
      4 + payloadSize // payload
    );
  }

  /**
   * Compress payload (placeholder for future implementation)
   */
  compress(data: ArrayBuffer): ArrayBuffer {
    // TODO: Implement compression (e.g., gzip, brotli)
    return data;
  }

  /**
   * Decompress payload (placeholder for future implementation)
   */
  decompress(data: ArrayBuffer): ArrayBuffer {
    // TODO: Implement decompression
    return data;
  }
}
