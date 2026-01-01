/**
 * TemplateEditor - Notification template editor
 */

import React, { useState } from 'react';
import { NotificationTemplate, NotificationChannelType, NotificationPriority } from '../types';

export interface TemplateEditorProps {
  template?: NotificationTemplate;
  onSave: (template: Partial<NotificationTemplate>) => void;
  onCancel?: () => void;
  className?: string;
}

export const TemplateEditor: React.FC<TemplateEditorProps> = ({
  template,
  onSave,
  onCancel,
  className = '',
}) => {
  const [name, setName] = useState(template?.name ?? '');
  const [description, setDescription] = useState(template?.description ?? '');
  const [type, setType] = useState(template?.type ?? '');
  const [subject, setSubject] = useState(template?.channels[0]?.subject ?? '');
  const [body, setBody] = useState(template?.channels[0]?.body ?? '');
  const [html, setHtml] = useState(template?.channels[0]?.html ?? '');

  const handleSave = () => {
    onSave({
      ...template,
      name,
      description,
      type,
      channels: [
        {
          type: NotificationChannelType.EMAIL,
          subject,
          body,
          html,
        },
      ],
      enabled: true,
    });
  };

  return (
    <div className={`template-editor ${className}`}>
      <h2>{template ? 'Edit Template' : 'Create Template'}</h2>

      <div className="form-group">
        <label>Template Name</label>
        <input type="text" value={name} onChange={e => setName(e.target.value)} />
      </div>

      <div className="form-group">
        <label>Description</label>
        <textarea value={description} onChange={e => setDescription(e.target.value)} />
      </div>

      <div className="form-group">
        <label>Subject</label>
        <input type="text" value={subject} onChange={e => setSubject(e.target.value)} />
      </div>

      <div className="form-group">
        <label>Body</label>
        <textarea
          value={body}
          onChange={e => setBody(e.target.value)}
          rows={6}
          placeholder="Use {{variable}} for dynamic content"
        />
      </div>

      <div className="form-group">
        <label>HTML (optional)</label>
        <textarea value={html} onChange={e => setHtml(e.target.value)} rows={10} />
      </div>

      <div className="actions">
        <button onClick={handleSave} className="primary">Save</button>
        {onCancel && <button onClick={onCancel} className="secondary">Cancel</button>}
      </div>
    </div>
  );
};

export default TemplateEditor;
