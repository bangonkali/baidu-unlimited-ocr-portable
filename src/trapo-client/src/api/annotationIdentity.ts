export interface AnnotationIdentity {
  annotation_id?: string;
  region_id: string;
}

export function annotationIdOf(value: AnnotationIdentity) {
  return value.annotation_id || value.region_id;
}

export function annotationDomId(prefix: string, value: string) {
  return `${prefix}-${value}`;
}

export function annotationBoxDomId(annotationId: string) {
  return annotationDomId('annotation-box', annotationId);
}

export function annotationTextDomId(annotationId: string) {
  return annotationDomId('annotation-text', annotationId);
}
