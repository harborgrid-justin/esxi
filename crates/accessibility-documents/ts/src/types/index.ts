/**
 * Document Accessibility Checker Types
 * Comprehensive type definitions for PDF/UA, EPUB, and Office document accessibility
 */

export enum DocumentType {
  PDF = 'pdf',
  WORD = 'word',
  EXCEL = 'excel',
  POWERPOINT = 'powerpoint',
  EPUB = 'epub',
  UNKNOWN = 'unknown'
}

export enum AccessibilitySeverity {
  CRITICAL = 'critical',
  ERROR = 'error',
  WARNING = 'warning',
  INFO = 'info'
}

export enum WCAGLevel {
  A = 'A',
  AA = 'AA',
  AAA = 'AAA'
}

export enum PDFUARequirement {
  TAGGED = 'tagged',
  STRUCTURE = 'structure',
  METADATA = 'metadata',
  LANGUAGE = 'language',
  READING_ORDER = 'reading_order',
  ALTERNATIVE_TEXT = 'alternative_text',
  HEADINGS = 'headings',
  LISTS = 'lists',
  TABLES = 'tables',
  FORMS = 'forms',
  LINKS = 'links',
  COLOR_CONTRAST = 'color_contrast',
  SEMANTIC_STRUCTURE = 'semantic_structure'
}

export interface AccessibilityIssue {
  id: string;
  severity: AccessibilitySeverity;
  type: string;
  title: string;
  description: string;
  location?: IssueLocation;
  wcagCriteria?: string[];
  wcagLevel?: WCAGLevel;
  pdfuaRequirement?: PDFUARequirement;
  remediation?: RemediationSuggestion;
  affectedElements?: string[];
  pageNumber?: number;
  timestamp: Date;
}

export interface IssueLocation {
  page?: number;
  element?: string;
  xpath?: string;
  coordinates?: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  context?: string;
}

export interface RemediationSuggestion {
  action: string;
  description: string;
  steps: string[];
  automated: boolean;
  estimatedEffort: 'low' | 'medium' | 'high';
  priority: number;
  codeExample?: string;
  toolsRequired?: string[];
}

export interface TagStructure {
  type: string;
  role?: string;
  title?: string;
  alt?: string;
  actualText?: string;
  children?: TagStructure[];
  attributes?: Record<string, any>;
  id?: string;
  level?: number;
  isArtifact?: boolean;
}

export interface DocumentMetadata {
  title?: string;
  author?: string;
  subject?: string;
  keywords?: string[];
  language?: string;
  creator?: string;
  producer?: string;
  creationDate?: Date;
  modificationDate?: Date;
  pdfVersion?: string;
  tagged?: boolean;
  encrypted?: boolean;
  pageCount?: number;
  fileSize?: number;
}

export interface CheckerResult {
  documentType: DocumentType;
  fileName: string;
  fileSize: number;
  metadata: DocumentMetadata;
  issues: AccessibilityIssue[];
  structure?: TagStructure;
  readingOrder?: ReadingOrderItem[];
  complianceScore: ComplianceScore;
  summary: CheckerSummary;
  checkedAt: Date;
  checkDuration: number;
}

export interface ReadingOrderItem {
  id: string;
  type: string;
  content: string;
  page: number;
  order: number;
  boundingBox?: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  role?: string;
}

export interface ComplianceScore {
  overall: number;
  pdfua?: number;
  wcagA?: number;
  wcagAA?: number;
  wcagAAA?: number;
  passedChecks: number;
  failedChecks: number;
  warningChecks: number;
  totalChecks: number;
}

export interface CheckerSummary {
  totalIssues: number;
  criticalIssues: number;
  errorIssues: number;
  warningIssues: number;
  infoIssues: number;
  isPDFUA: boolean;
  isWCAGA: boolean;
  isWCAGAA: boolean;
  isWCAGAAA: boolean;
  hasTaggedStructure: boolean;
  hasMetadata: boolean;
  hasLanguage: boolean;
  hasAlternativeText: boolean;
  recommendedActions: string[];
}

export interface CheckerOptions {
  checkPDFUA?: boolean;
  checkWCAG?: boolean;
  wcagLevel?: WCAGLevel;
  includeWarnings?: boolean;
  checkReadingOrder?: boolean;
  checkColorContrast?: boolean;
  checkSemantics?: boolean;
  validateTags?: boolean;
  validateMetadata?: boolean;
  validateLanguage?: boolean;
  maxFileSize?: number;
  timeout?: number;
}

export interface PDFAnalysisResult {
  isTagged: boolean;
  version: string;
  pageCount: number;
  hasStructureTree: boolean;
  hasMarkInfo: boolean;
  language?: string;
  title?: string;
  metadata: DocumentMetadata;
  structureTree?: TagStructure;
  fonts: FontInfo[];
  images: ImageInfo[];
  forms: FormFieldInfo[];
  links: LinkInfo[];
  annotations: AnnotationInfo[];
  readingOrder: ReadingOrderItem[];
}

export interface FontInfo {
  name: string;
  type: string;
  embedded: boolean;
  subset: boolean;
  encoding?: string;
  unicodeMapping: boolean;
}

export interface ImageInfo {
  id: string;
  page: number;
  hasAltText: boolean;
  altText?: string;
  width: number;
  height: number;
  format?: string;
  isDecorative?: boolean;
  role?: string;
}

export interface FormFieldInfo {
  name: string;
  type: string;
  hasLabel: boolean;
  label?: string;
  required: boolean;
  page: number;
  tabIndex?: number;
  tooltipText?: string;
}

export interface LinkInfo {
  text: string;
  url?: string;
  page: number;
  hasDescription: boolean;
  isAccessible: boolean;
}

export interface AnnotationInfo {
  type: string;
  page: number;
  contents?: string;
  hasAlternativeDescription: boolean;
}

export interface OfficeAnalysisResult {
  documentType: DocumentType;
  hasStyles: boolean;
  hasHeadings: boolean;
  hasTableOfContents: boolean;
  hasAltText: boolean;
  language?: string;
  metadata: DocumentMetadata;
  headings: HeadingInfo[];
  images: ImageInfo[];
  tables: TableInfo[];
  lists: ListInfo[];
  links: LinkInfo[];
}

export interface HeadingInfo {
  level: number;
  text: string;
  style?: string;
  page?: number;
  order: number;
  isAccessible: boolean;
}

export interface TableInfo {
  id: string;
  rowCount: number;
  columnCount: number;
  hasHeaders: boolean;
  hasCaption: boolean;
  caption?: string;
  headerCells: string[];
  page?: number;
  isAccessible: boolean;
}

export interface ListInfo {
  type: 'ordered' | 'unordered';
  itemCount: number;
  nested: boolean;
  page?: number;
  isAccessible: boolean;
}

export interface EPUBAnalysisResult {
  version: string;
  hasNavigation: boolean;
  hasSemantics: boolean;
  hasAccessibilityMetadata: boolean;
  metadata: DocumentMetadata;
  contentDocuments: ContentDocumentInfo[];
  navigation: NavigationInfo[];
  mediaOverlays?: MediaOverlayInfo[];
}

export interface ContentDocumentInfo {
  href: string;
  title?: string;
  hasHeadings: boolean;
  hasLandmarks: boolean;
  hasAltText: boolean;
  language?: string;
}

export interface NavigationInfo {
  type: 'toc' | 'page-list' | 'landmarks';
  items: NavigationItem[];
}

export interface NavigationItem {
  title: string;
  href: string;
  level: number;
  children?: NavigationItem[];
}

export interface MediaOverlayInfo {
  textRef: string;
  audioRef: string;
  duration?: number;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  code: string;
  message: string;
  element?: string;
  location?: IssueLocation;
}

export interface ValidationWarning {
  code: string;
  message: string;
  element?: string;
  location?: IssueLocation;
}

export interface CheckerProgress {
  stage: CheckerStage;
  progress: number;
  message: string;
  currentFile?: string;
}

export enum CheckerStage {
  UPLOADING = 'uploading',
  PARSING = 'parsing',
  ANALYZING_STRUCTURE = 'analyzing_structure',
  VALIDATING_TAGS = 'validating_tags',
  CHECKING_METADATA = 'checking_metadata',
  CHECKING_IMAGES = 'checking_images',
  CHECKING_FORMS = 'checking_forms',
  CHECKING_READING_ORDER = 'checking_reading_order',
  GENERATING_REPORT = 'generating_report',
  COMPLETE = 'complete',
  ERROR = 'error'
}
