/**
 * Comment Thread Component
 * Displays and manages inline comments
 */

import React, { useState } from 'react';
import { CommentThread as CommentThreadType, Comment, CommentStatus } from '../types';

export interface CommentThreadProps {
  thread: CommentThreadType;
  currentUserId: string;
  onAddComment?: (threadId: string, content: string) => void;
  onUpdateComment?: (commentId: string, content: string) => void;
  onDeleteComment?: (commentId: string) => void;
  onResolveThread?: (threadId: string) => void;
  onReopenThread?: (threadId: string) => void;
  onReact?: (commentId: string, emoji: string) => void;
  className?: string;
}

export const CommentThread: React.FC<CommentThreadProps> = ({
  thread,
  currentUserId,
  onAddComment,
  onUpdateComment,
  onDeleteComment,
  onResolveThread,
  onReopenThread,
  onReact,
  className = '',
}) => {
  const [newComment, setNewComment] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editContent, setEditContent] = useState('');
  const [showReactions, setShowReactions] = useState<string | null>(null);

  const handleAddComment = () => {
    if (newComment.trim() && onAddComment) {
      onAddComment(thread.id, newComment);
      setNewComment('');
    }
  };

  const handleUpdateComment = (commentId: string) => {
    if (editContent.trim() && onUpdateComment) {
      onUpdateComment(commentId, editContent);
      setEditingId(null);
      setEditContent('');
    }
  };

  const handleStartEdit = (comment: Comment) => {
    setEditingId(comment.id);
    setEditContent(comment.content);
  };

  const handleCancelEdit = () => {
    setEditingId(null);
    setEditContent('');
  };

  const formatTimestamp = (date: Date): string => {
    return new Date(date).toLocaleString();
  };

  const reactions = ['👍', '❤️', '😄', '🎉', '🤔'];

  return (
    <div className={`bg-white border border-gray-200 rounded-lg shadow ${className}`}>
      {/* Thread Header */}
      <div className="border-b border-gray-200 px-4 py-3 bg-gray-50">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <span className="text-sm font-medium text-gray-900">
              {thread.comments.length} Comment{thread.comments.length !== 1 ? 's' : ''}
            </span>
            <span
              className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${
                thread.status === CommentStatus.RESOLVED
                  ? 'bg-green-100 text-green-800'
                  : 'bg-blue-100 text-blue-800'
              }`}
            >
              {thread.status}
            </span>
          </div>

          <div className="flex space-x-2">
            {thread.status === CommentStatus.OPEN && onResolveThread && (
              <button
                onClick={() => onResolveThread(thread.id)}
                className="px-3 py-1 text-sm bg-green-600 text-white rounded hover:bg-green-700 transition-colors"
              >
                Resolve
              </button>
            )}
            {thread.status === CommentStatus.RESOLVED && onReopenThread && (
              <button
                onClick={() => onReopenThread(thread.id)}
                className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
              >
                Reopen
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Comments List */}
      <div className="divide-y divide-gray-200 max-h-96 overflow-y-auto">
        {thread.comments.map((comment) => (
          <div key={comment.id} className="px-4 py-3">
            <div className="flex items-start space-x-3">
              <div className="flex-shrink-0">
                <div className="w-8 h-8 bg-gray-300 rounded-full flex items-center justify-center text-sm font-medium text-gray-700">
                  {comment.authorName.charAt(0).toUpperCase()}
                </div>
              </div>

              <div className="flex-1 min-w-0">
                <div className="flex items-center space-x-2">
                  <span className="text-sm font-medium text-gray-900">
                    {comment.authorName}
                  </span>
                  <span className="text-xs text-gray-500">
                    {formatTimestamp(comment.createdAt)}
                  </span>
                  {comment.createdAt.getTime() !== comment.updatedAt.getTime() && (
                    <span className="text-xs text-gray-400">(edited)</span>
                  )}
                </div>

                {editingId === comment.id ? (
                  <div className="mt-2 space-y-2">
                    <textarea
                      value={editContent}
                      onChange={(e) => setEditContent(e.target.value)}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      rows={3}
                    />
                    <div className="flex space-x-2">
                      <button
                        onClick={() => handleUpdateComment(comment.id)}
                        className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
                      >
                        Save
                      </button>
                      <button
                        onClick={handleCancelEdit}
                        className="px-3 py-1 text-sm bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                ) : (
                  <>
                    <p className="mt-1 text-sm text-gray-700">{comment.content}</p>

                    {/* Reactions */}
                    {comment.reactions && comment.reactions.size > 0 && (
                      <div className="mt-2 flex flex-wrap gap-1">
                        {Array.from(comment.reactions.entries()).map(([emoji, userIds]) => (
                          <button
                            key={emoji}
                            onClick={() => onReact?.(comment.id, emoji)}
                            className={`inline-flex items-center px-2 py-0.5 rounded text-sm ${
                              userIds.includes(currentUserId)
                                ? 'bg-blue-100 border border-blue-300'
                                : 'bg-gray-100 hover:bg-gray-200'
                            }`}
                          >
                            <span>{emoji}</span>
                            <span className="ml-1 text-xs text-gray-600">
                              {userIds.length}
                            </span>
                          </button>
                        ))}
                      </div>
                    )}

                    {/* Actions */}
                    <div className="mt-2 flex items-center space-x-3 text-xs">
                      <button
                        onClick={() => setShowReactions(showReactions === comment.id ? null : comment.id)}
                        className="text-gray-500 hover:text-gray-700"
                      >
                        React
                      </button>
                      {comment.authorId === currentUserId && (
                        <>
                          <button
                            onClick={() => handleStartEdit(comment)}
                            className="text-gray-500 hover:text-gray-700"
                          >
                            Edit
                          </button>
                          <button
                            onClick={() => onDeleteComment?.(comment.id)}
                            className="text-red-500 hover:text-red-700"
                          >
                            Delete
                          </button>
                        </>
                      )}
                    </div>

                    {/* Reaction Picker */}
                    {showReactions === comment.id && (
                      <div className="mt-2 flex space-x-1">
                        {reactions.map((emoji) => (
                          <button
                            key={emoji}
                            onClick={() => {
                              onReact?.(comment.id, emoji);
                              setShowReactions(null);
                            }}
                            className="px-2 py-1 text-lg hover:bg-gray-100 rounded"
                          >
                            {emoji}
                          </button>
                        ))}
                      </div>
                    )}
                  </>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Add Comment */}
      {thread.status === CommentStatus.OPEN && onAddComment && (
        <div className="border-t border-gray-200 px-4 py-3">
          <div className="flex space-x-3">
            <div className="flex-shrink-0">
              <div className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center text-sm font-medium text-white">
                You
              </div>
            </div>
            <div className="flex-1">
              <textarea
                value={newComment}
                onChange={(e) => setNewComment(e.target.value)}
                placeholder="Add a comment..."
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                rows={2}
              />
              <div className="mt-2 flex justify-end">
                <button
                  onClick={handleAddComment}
                  disabled={!newComment.trim()}
                  className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors text-sm"
                >
                  Comment
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
