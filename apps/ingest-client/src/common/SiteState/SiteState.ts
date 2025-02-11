import {
  SectionReference,
  SessionReference,
  VisitorReference,
} from "../Event/Event";

export enum SiteStateRepositoryError {
  InternalError,
  MalformedData,
  NotFound,
}

export interface SiteState {
  visitor: VisitorReference | undefined;
  session: SessionReference | undefined;
  section: SectionReference | undefined;
}

export type SiteStateRepositoryResult = SiteState | SiteStateRepositoryError;

export function isSiteStateRepositoryError(
  result: SiteStateRepositoryResult,
): result is SiteStateRepositoryError {
  return typeof result == "number";
}

export interface SiteStateRepository {
  getSiteState(): SiteStateRepositoryResult;
  setSiteState(state: SiteState): SiteStateRepositoryResult;
  clearSiteState(): SiteStateRepositoryResult;
}
