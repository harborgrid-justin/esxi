/**
 * Document Accessibility Checker Component
 * Main component for checking document accessibility
 */

import React, { useState } from 'react';
import type { CheckerResult, CheckerOptions } from '../../types/index.js';
import { useDocumentChecker } from '../../hooks/useDocumentChecker.js';
import { FileUpload } from './FileUpload.js';
import { AnalysisProgress } from './AnalysisProgress.js';
import { CheckerResults } from '../Results/CheckerResults.js';

export interface DocumentCheckerProps {
  options?: CheckerOptions;
  onComplete?: (result: CheckerResult) => void;
  onError?: (error: Error) => void;
  className?: string;
}

export function DocumentChecker({
  options,
  onComplete,
  onError,
  className = ''
}: DocumentCheckerProps): JSX.Element {
  const { checkDocument, reset, isChecking, progress, result, error } = useDocumentChecker(options);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);

  const handleFileSelect = async (file: File) => {
    setSelectedFile(file);

    try {
      const result = await checkDocument(file);
      onComplete?.(result);
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Check failed');
      onError?.(error);
    }
  };

  const handleReset = () => {
    setSelectedFile(null);
    reset();
  };

  return (
    <div className={`document-checker ${className}`}>
      <div className="document-checker__header">
        <h1>Document Accessibility Checker</h1>
        <p>
          Upload your PDF, Word, Excel, PowerPoint, or EPUB document to check for accessibility
          compliance with WCAG 2.1/2.2 and PDF/UA standards.
        </p>
      </div>

      {!selectedFile && !isChecking && !result && (
        <FileUpload onFileSelect={handleFileSelect} />
      )}

      {isChecking && progress && (
        <AnalysisProgress progress={progress} fileName={selectedFile?.name} />
      )}

      {error && (
        <div className="document-checker__error" role="alert">
          <h2>Error</h2>
          <p>{error.message}</p>
          <button onClick={handleReset}>Try Again</button>
        </div>
      )}

      {result && (
        <div className="document-checker__results">
          <CheckerResults result={result} />
          <div className="document-checker__actions">
            <button onClick={handleReset} className="button--primary">
              Check Another Document
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
