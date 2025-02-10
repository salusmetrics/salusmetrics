import {
  EventType,
  isEventReference,
  SectionReference,
  SessionReference,
  VisitorReference,
} from "../Event/Event";
import {
  EventConfiguration,
  EventConfigurationState,
  validateEventConfiguration,
} from "../Event/EventConfiguration";
import {
  SiteState,
  SiteStateRepository,
  SiteStateRepositoryError,
  SiteStateRepositoryResult,
} from "./SiteState";

const PREFIX: string = "SALUS";
const VISITOR_SUFFIX: string = "VISITOR";
const SESSION_SUFFIX: string = "SESSION";

export class WebStorageSiteStateRepository implements SiteStateRepository {
  private api_key: string;
  private section: SectionReference | undefined;

  constructor(config: EventConfiguration) {
    if (
      validateEventConfiguration(config) ==
      EventConfigurationState.InvalidApiKey
    ) {
      throw new TypeError("invalid empty api_key");
    }
    this.api_key = config.api_key;
    this.section = undefined;
  }

  clearSiteState(): SiteStateRepositoryResult {
    this.clearSection();
    this.clearSession();
    this.clearVisitor();
    return { visitor: undefined, session: undefined, section: undefined };
  }

  getSiteState(): SiteStateRepositoryResult {
    const visitorResult = this.getVisitor();
    let visitor: VisitorReference | undefined = undefined;
    if (typeof visitorResult == "number") {
      return visitorResult;
    } else {
      visitor = visitorResult;
    }

    const sessionResult = this.getSession();
    let session: SessionReference | undefined = undefined;
    if (typeof sessionResult == "number") {
      if (
        sessionResult == SiteStateRepositoryError.MalformedData ||
        sessionResult == SiteStateRepositoryError.InternalError
      ) {
        return sessionResult;
      } else {
        return { visitor, session: undefined, section: undefined };
      }
    } else {
      session = sessionResult;
    }

    const sectionResult = this.getSection();
    let section: SectionReference | undefined = undefined;
    if (typeof sectionResult == "number") {
      if (
        sectionResult == SiteStateRepositoryError.MalformedData ||
        sectionResult == SiteStateRepositoryError.InternalError
      ) {
        return sectionResult;
      }
    } else {
      section = sectionResult;
    }

    return {
      visitor,
      session,
      section,
    };
  }

  setSiteState(state: SiteState): SiteStateRepositoryResult {
    const section = state.section;
    if (typeof section == "undefined") {
      this.clearSection();
    } else {
      const sectionResult = this.setSection(section);
      if (!isEventReference(sectionResult)) {
        return sectionResult;
      }
    }
    const session = state.session;
    if (typeof session == "undefined") {
      this.clearSession();
    } else {
      const sessionResult = this.setSession(session);
      if (!isEventReference(sessionResult)) {
        return sessionResult;
      }
    }
    const visitor = state.visitor;
    if (typeof visitor == "undefined") {
      this.clearVisitor();
    } else {
      const visitorResult = this.setVisitor(visitor);
      if (!isEventReference(visitorResult)) {
        return visitorResult;
      }
    }
    return state;
  }

  private getVisitor(): VisitorReference | SiteStateRepositoryError {
    const key: string = [PREFIX, VISITOR_SUFFIX, this.api_key].join("_");
    const id = localStorage.getItem(key);
    if (id == null) {
      return SiteStateRepositoryError.NotFound;
    }
    const visitor = { id, event_type: EventType.Visitor };
    if (!isEventReference(visitor)) {
      return SiteStateRepositoryError.MalformedData;
    }

    return visitor as VisitorReference;
  }

  private setVisitor(
    visitor: VisitorReference,
  ): VisitorReference | SiteStateRepositoryError {
    const key: string = [PREFIX, VISITOR_SUFFIX, this.api_key].join("_");
    try {
      localStorage.setItem(key, visitor.id);
    } catch (e) {
      return SiteStateRepositoryError.InternalError;
    }
    return visitor;
  }

  private clearVisitor(): void {
    const key: string = [PREFIX, VISITOR_SUFFIX, this.api_key].join("_");
    localStorage.removeItem(key);
  }

  private getSession(): SessionReference | SiteStateRepositoryError {
    const key: string = [PREFIX, SESSION_SUFFIX, this.api_key].join("_");
    const id = sessionStorage.getItem(key);
    if (id == null) {
      return SiteStateRepositoryError.NotFound;
    }
    const session = { id, event_type: EventType.Session };
    if (!isEventReference(session)) {
      return SiteStateRepositoryError.MalformedData;
    }

    return session as SessionReference;
  }

  private setSession(
    session: SessionReference,
  ): SessionReference | SiteStateRepositoryError {
    const key: string = [PREFIX, SESSION_SUFFIX, this.api_key].join("_");
    try {
      sessionStorage.setItem(key, session.id);
    } catch (e) {
      return SiteStateRepositoryError.InternalError;
    }
    return session;
  }

  private clearSession(): void {
    const key: string = [PREFIX, SESSION_SUFFIX, this.api_key].join("_");
    localStorage.removeItem(key);
  }

  private getSection(): SectionReference | SiteStateRepositoryError {
    return this.section == undefined
      ? SiteStateRepositoryError.NotFound
      : this.section;
  }

  private setSection(
    section: SectionReference,
  ): SectionReference | SiteStateRepositoryError {
    this.section = section;
    return this.section;
  }

  private clearSection(): void {
    this.section = undefined;
  }
}
