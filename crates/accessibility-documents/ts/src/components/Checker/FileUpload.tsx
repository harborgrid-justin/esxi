/**
 * Accessible File Upload Component
 * Drag-and-drop file upload with keyboard support
 */

import React, { useRef, useState, useCallback } from 'react';

export interface FileUploadProps {
  onFileSelect: (file: File) => void;
  acceptedTypes?: string[];
  maxSizeMB?: number;
  className?: string;
}

export function FileUpload({
  onFileSelect,
  acceptedTypes = ['.pdf', '.docx', '.xlsx', '.pptx', '.epub'],
  maxSizeMB = 50,
  className = ''
}: FileUploadProps): JSX.Element {
  const [isDragging, setIsDragging] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const acceptedMimeTypes = {
    '.pdf': 'application/pdf',
    '.docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    '.xlsx': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    '.pptx': 'application/vnd.openxmlformats-officedocument.presentationml.presentation',
    '.epub': 'application/epub+zip'
  };

  const validateFile = (file: File): string | null => {
    // Check file size
    const maxSizeBytes = maxSizeMB * 1024 * 1024;
    if (file.size > maxSizeBytes) {
      return `File size exceeds ${maxSizeMB}MB limit`;
    }

    // Check file type
    const extension = '.' + file.name.split('.').pop()?.toLowerCase();
    if (!acceptedTypes.includes(extension)) {
      return `File type not supported. Accepted types: ${acceptedTypes.join(', ')}`;
    }

    return null;
  };

  const handleFile = useCallback((file: File) => {
    const error = validateFile(file);
    if (error) {
      setError(error);
      return;
    }

    setError(null);
    onFileSelect(file);
  }, [onFileSelect, maxSizeMB, acceptedTypes]);

  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) {
      handleFile(files[0]);
    }
  };

  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files && files.length > 0) {
      handleFile(files[0]);
    }
  };

  const handleClick = () => {
    fileInputRef.current?.click();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleClick();
    }
  };

  return (
    <div className={`file-upload ${className}`}>
      <div
        className={`file-upload__dropzone ${isDragging ? 'file-upload__dropzone--dragging' : ''}`}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
        onClick={handleClick}
        onKeyDown={handleKeyDown}
        role="button"
        tabIndex={0}
        aria-label="Upload document for accessibility check. Click or drag and drop a file."
      >
        <div className="file-upload__icon" aria-hidden="true">
          <svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M32 8L32 40M32 8L20 20M32 8L44 20M8 48L8 52C8 54.2091 9.79086 56 12 56L52 56C54.2091 56 56 54.2091 56 52L56 48"
                  stroke="currentColor" strokeWidth="4" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </div>
        <div className="file-upload__text">
          <p className="file-upload__title">
            Drop your document here or click to browse
          </p>
          <p className="file-upload__subtitle">
            Supported formats: {acceptedTypes.join(', ')}
          </p>
          <p className="file-upload__subtitle">
            Maximum file size: {maxSizeMB}MB
          </p>
        </div>
        <input
          ref={fileInputRef}
          type="file"
          accept={acceptedTypes.join(',')}
          onChange={handleFileInputChange}
          className="file-upload__input"
          aria-hidden="true"
          tabIndex={-1}
        />
      </div>

      {error && (
        <div className="file-upload__error" role="alert">
          <p>{error}</p>
        </div>
      )}

      <div className="file-upload__info">
        <h3>What we check:</h3>
        <ul>
          <li>PDF/UA compliance for PDF documents</li>
          <li>WCAG 2.1 Level A, AA, AAA criteria</li>
          <li>Document structure and semantics</li>
          <li>Alternative text for images</li>
          <li>Heading hierarchy</li>
          <li>Table accessibility</li>
          <li>Form field labels</li>
          <li>Reading order</li>
          <li>Color contrast</li>
          <li>Language specification</li>
        </ul>
      </div>
    </div>
  );
}
