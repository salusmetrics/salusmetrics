import {
  Click,
  Section,
  SectionReference,
  Session,
  SessionReference,
  Visitor,
  VisitorReference,
} from "../Event/Event";
import {
  EventRegistry,
  EventRegistryError,
  EventRegistryResult,
} from "../Event/EventRegistry";
import {
  EventPublisher,
  EventPublishError,
  EventPublishResult,
  isEventPublishResultError,
} from "../EventPublisher/EventPublisher";
import { ToPublishEvent } from "../EventPublisher/PublishEvent";
import {
  isSiteStateRepositoryError,
  SiteState,
  SiteStateRepository,
  SiteStateRepositoryResult,
} from "../SiteState/SiteState";

export class EventManager implements EventRegistry {
  private publisher: EventPublisher;
  private siteStateRepository: SiteStateRepository;
  private eventQueue: ToPublishEvent[];
  private siteStateWriteBuffer: SiteState | undefined;

  constructor(
    publisher: EventPublisher,
    siteStateRepository: SiteStateRepository,
  ) {
    this.publisher = publisher;
    this.siteStateRepository = siteStateRepository;
    this.eventQueue = [];
    this.siteStateWriteBuffer = undefined;
  }

  private getSiteState(): SiteState {
    if (this.siteStateWriteBuffer != undefined) {
      return this.siteStateWriteBuffer;
    }
    const repoValue = this.siteStateRepository.getSiteState();
    if (isSiteStateRepositoryError(repoValue)) {
      this.siteStateWriteBuffer = {
        visitor: undefined,
        session: undefined,
        section: undefined,
      };
    } else {
      this.siteStateWriteBuffer = repoValue;
    }
    return this.siteStateWriteBuffer;
  }

  private writeSiteState(): SiteStateRepositoryResult {
    let result: SiteStateRepositoryResult;
    if (this.siteStateWriteBuffer != undefined) {
      result = this.siteStateRepository.setSiteState(this.siteStateWriteBuffer);
    } else {
      result = this.siteStateRepository.clearSiteState();
    }
    this.siteStateWriteBuffer = undefined;
    return result;
  }

  private setVisitorReference(visitor: VisitorReference | undefined): void {
    this.siteStateWriteBuffer = {
      visitor,
      session: undefined,
      section: undefined,
    };
  }

  private setSessionReference(session: SessionReference | undefined): void {
    this.siteStateWriteBuffer = {
      visitor: this.getSiteState().visitor,
      session,
      section: undefined,
    };
  }

  private setSectionReference(section: SectionReference | undefined): void {
    this.siteStateWriteBuffer = {
      visitor: this.getSiteState().visitor,
      session: this.getSiteState().session,
      section,
    };
  }

  private flush(): Promise<EventRegistryResult> {
    const siteStateSetResult = this.writeSiteState();
    if (isSiteStateRepositoryError(siteStateSetResult)) {
      return new Promise((resolve) =>
        resolve(EventRegistryError.InternalError),
      );
    }
    const promise = this.publisher
      .publish(this.eventQueue.map((e) => e.toPublishEvent()))
      .then((result) =>
        this.mapEventPublishResultToEventRegistryResult(result),
      );
    this.eventQueue = [];
    return promise;
  }

  private mapEventPublishResultToEventRegistryResult(
    pubRes: EventPublishResult,
  ): EventRegistryResult {
    if (isEventPublishResultError(pubRes)) {
      if (pubRes == EventPublishError.BadRequest) {
        return EventRegistryError.BadRequest;
      } else if (pubRes == EventPublishError.ConfigurationError) {
        return EventRegistryError.ConfigurationError;
      } else {
        return EventRegistryError.InternalError;
      }
    } else {
      return pubRes;
    }
  }

  registerVisitor(): Promise<EventRegistryResult> {
    this.createVisitor();
    return this.flush();
  }

  deregisterVisitor(): Promise<EventRegistryResult> {
    this.setVisitorReference(undefined);
    return this.flush();
  }

  private createVisitor(): Visitor {
    const visitor = new Visitor();
    this.setVisitorReference(visitor);
    this.eventQueue.push(visitor);
    return visitor;
  }

  private getOrCreateVisitorReference(): VisitorReference {
    const siteStateVisitor = this.getSiteState().visitor;
    if (siteStateVisitor != undefined) {
      return siteStateVisitor;
    }
    return this.createVisitor();
  }

  registerSession(): Promise<EventRegistryResult> {
    this.createSession();
    return this.flush();
  }

  deregisterSession(): Promise<EventRegistryResult> {
    this.setSessionReference(undefined);
    return this.flush();
  }

  private createSession(): Session {
    const visitor = this.getOrCreateVisitorReference();
    const session = new Session(visitor);
    this.setSessionReference(session);
    this.eventQueue.push(session);
    return session;
  }

  private getOrCreateSessionReference(): SessionReference {
    const siteStateSession = this.getSiteState().session;
    if (siteStateSession != undefined) {
      return siteStateSession;
    }
    return this.createSession();
  }

  registerSection(): Promise<EventRegistryResult> {
    this.createSection();
    return this.flush();
  }

  deregisterSection(): Promise<EventRegistryResult> {
    this.setSectionReference(undefined);
    return this.flush();
  }

  private createSection(): Section {
    const session = this.getOrCreateSessionReference();
    const section = new Section(session);
    this.setSectionReference(section);
    this.eventQueue.push(section);
    return section;
  }

  private getOrCreateSectionReference(): SectionReference {
    const siteStateSection = this.getSiteState().section;
    if (siteStateSection != undefined) {
      return siteStateSection;
    }
    return this.createSection();
  }

  registerClick(): Promise<EventRegistryResult> {
    this.createClick();
    return this.flush();
  }

  private createClick(): Click {
    const section = this.getOrCreateSectionReference();
    const click = new Click(section);
    this.eventQueue.push(click);
    return click;
  }
}
