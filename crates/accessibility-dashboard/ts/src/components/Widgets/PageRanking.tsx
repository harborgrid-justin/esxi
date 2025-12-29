/**
 * Page Ranking Widget
 * Displays pages ranked by number of issues
 */

import React from 'react';
import { format } from 'date-fns';
import clsx from 'clsx';
import type { PageCompliance } from '../../types';
import { getScoreColor } from '../../utils/calculations';

export interface PageRankingProps {
  pages: PageCompliance[];
  onPageClick?: (page: PageCompliance) => void;
  maxItems?: number;
  showScore?: boolean;
}

export const PageRanking: React.FC<PageRankingProps> = ({
  pages,
  onPageClick,
  maxItems = 10,
  showScore = true,
}) => {
  const displayPages = pages.slice(0, maxItems);

  if (pages.length === 0) {
    return (
      <div
        className="text-center py-8 bg-gray-50 rounded-lg border border-gray-200"
        role="status"
      >
        <p className="text-gray-500 text-sm">No pages analyzed</p>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg border border-gray-200 shadow-sm">
      <div className="px-4 py-3 border-b border-gray-200 bg-gray-50">
        <h3 className="text-sm font-semibold text-gray-900">
          Pages by Issue Count
        </h3>
        <p className="text-xs text-gray-500 mt-1">
          Showing top {displayPages.length} of {pages.length} pages
        </p>
      </div>

      <ul className="divide-y divide-gray-200" role="list">
        {displayPages.map((page, index) => {
          const scoreColor = getScoreColor(page.score.overall);
          const rank = index + 1;

          return (
            <li
              key={page.url}
              className={clsx(
                'px-4 py-4 hover:bg-gray-50 transition-colors',
                onPageClick && 'cursor-pointer'
              )}
              onClick={() => onPageClick?.(page)}
              tabIndex={onPageClick ? 0 : undefined}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  onPageClick?.(page);
                }
              }}
              role={onPageClick ? 'button' : undefined}
            >
              <div className="flex items-start gap-4">
                {/* Rank */}
                <div
                  className={clsx(
                    'flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center font-bold text-sm',
                    rank === 1 && 'bg-amber-100 text-amber-700',
                    rank === 2 && 'bg-gray-200 text-gray-700',
                    rank === 3 && 'bg-orange-100 text-orange-700',
                    rank > 3 && 'bg-gray-100 text-gray-600'
                  )}
                  aria-label={`Rank ${rank}`}
                >
                  {rank}
                </div>

                {/* Content */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                      <h4 className="text-sm font-medium text-gray-900 truncate">
                        {page.title}
                      </h4>
                      <p className="text-xs text-gray-500 truncate mt-0.5">
                        {page.url}
                      </p>
                    </div>

                    {showScore && (
                      <div className="flex-shrink-0 text-right">
                        <div
                          className="text-lg font-bold"
                          style={{ color: scoreColor }}
                          aria-label={`Compliance score: ${page.score.overall}%`}
                        >
                          {page.score.overall}%
                        </div>
                      </div>
                    )}
                  </div>

                  {/* Metrics */}
                  <div className="mt-2 flex items-center gap-4 text-xs">
                    <span className="text-gray-600">
                      <span className="font-medium text-gray-900">
                        {page.issueCount}
                      </span>{' '}
                      {page.issueCount === 1 ? 'issue' : 'issues'}
                    </span>

                    {page.criticalIssues > 0 && (
                      <span className="text-red-600">
                        <span className="font-medium">
                          {page.criticalIssues}
                        </span>{' '}
                        critical
                      </span>
                    )}

                    <span className="text-gray-500">
                      Last scanned: {format(page.lastScanned, 'MMM d')}
                    </span>
                  </div>

                  {/* Progress bar */}
                  <div className="mt-2">
                    <div className="w-full bg-gray-200 rounded-full h-1.5">
                      <div
                        className="h-1.5 rounded-full transition-all"
                        style={{
                          width: `${page.score.overall}%`,
                          backgroundColor: scoreColor,
                        }}
                        role="progressbar"
                        aria-valuenow={page.score.overall}
                        aria-valuemin={0}
                        aria-valuemax={100}
                        aria-label={`Compliance: ${page.score.overall}%`}
                      />
                    </div>
                  </div>
                </div>
              </div>
            </li>
          );
        })}
      </ul>

      {pages.length > maxItems && (
        <div className="px-4 py-3 border-t border-gray-200 bg-gray-50 text-center">
          <p className="text-xs text-gray-500">
            {pages.length - maxItems} more pages not shown
          </p>
        </div>
      )}
    </div>
  );
};
