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
  RegisterEventError,
  RegisterEventResult,
} from "../Event/EventRegistry";
import { EventPublisher } from "../EventPublisher/EventPublisher";
import { ToPublishEvent } from "../EventPublisher/PublishEvent";
import {
  isSiteStateRepositoryError,
  SiteState,
  SiteStateRepository,
} from "../SiteState/SiteState";

export class EventManager implements EventRegistry {
  private publisher: EventPublisher;
  private siteStateRepository: SiteStateRepository;
  private eventQueue: ToPublishEvent[];
  private siteStateWriteBuffer: SiteState;

  constructor(
    publisher: EventPublisher,
    siteStateRepository: SiteStateRepository,
  ) {
    this.publisher = publisher;
    this.siteStateRepository = siteStateRepository;
    this.eventQueue = [];
    this.siteStateWriteBuffer = {
      visitor: undefined,
      session: undefined,
      section: undefined,
    };
  }

  private flush(): RegisterEventResult {
    const siteStateSetResult = this.siteStateRepository.setSiteState(
      this.siteStateWriteBuffer,
    );
    if (isSiteStateRepositoryError(siteStateSetResult)) {
      return RegisterEventError.InternalError;
    }
    this.publisher.publish(this.eventQueue);
    this.eventQueue = [];
  }

  registerVisitor(): RegisterEventResult {
    this.createVisitor();
    return this.flush();
  }

  deregisterVisitor(): RegisterEventResult {
    this.siteStateWriteBuffer = {
      visitor: undefined,
      session: undefined,
      section: undefined,
    };
    return this.flush();
  }

  private createVisitor(): Visitor {
    const visitor = new Visitor();
    this.siteStateWriteBuffer = {
      visitor,
      session: undefined,
      section: undefined,
    };
    this.eventQueue.push(visitor);
    return visitor;
  }

  private getOrCreateVisitorReference(): VisitorReference {
    const siteStateResult = this.siteStateRepository.getSiteState();
    if (
      !isSiteStateRepositoryError(siteStateResult) &&
      siteStateResult.visitor != undefined
    ) {
      this.siteStateWriteBuffer = siteStateResult;
      return siteStateResult.visitor;
    }
    return this.createVisitor();
  }

  registerSession(): RegisterEventResult {
    this.createSession();
    return this.flush();
  }

  deregisterSession(): RegisterEventResult {
    let siteState = this.siteStateRepository.getSiteState();
    if (isSiteStateRepositoryError(siteState)) {
      this.siteStateWriteBuffer = {
        visitor: undefined,
        session: undefined,
        section: undefined,
      };
    } else {
      this.siteStateWriteBuffer = {
        ...siteState,
        session: undefined,
        section: undefined,
      };
    }
    return this.flush();
  }

  private createSession(): Session {
    const visitor = this.getOrCreateVisitorReference();
    const session = new Session(visitor);
    this.siteStateWriteBuffer.session = session;
    this.siteStateWriteBuffer.section = undefined;
    this.eventQueue.push(session);
    return session;
  }

  private getOrCreateSessionReference(): SessionReference {
    this.getOrCreateVisitorReference();
    const siteStateResult = this.siteStateRepository.getSiteState();
    if (
      !isSiteStateRepositoryError(siteStateResult) &&
      siteStateResult.session != undefined
    ) {
      this.siteStateWriteBuffer = siteStateResult;
      return siteStateResult.session;
    }
    return this.createSession();
  }

  registerSection(): RegisterEventResult {
    this.createSection();
    return this.flush();
  }

  deregisterSection(): RegisterEventResult {
    let siteState = this.siteStateRepository.getSiteState();
    if (isSiteStateRepositoryError(siteState)) {
      this.siteStateWriteBuffer = {
        visitor: undefined,
        session: undefined,
        section: undefined,
      };
    } else {
      this.siteStateWriteBuffer = { ...siteState, section: undefined };
    }
    return this.flush();
  }

  private createSection(): Section {
    const session = this.getOrCreateSessionReference();
    const section = new Section(session);
    this.siteStateWriteBuffer.section = section;
    this.eventQueue.push(section);
    return section;
  }

  private getOrCreateSectionReference(): SectionReference {
    this.getOrCreateSessionReference();
    const siteStateResult = this.siteStateRepository.getSiteState();
    if (
      !isSiteStateRepositoryError(siteStateResult) &&
      siteStateResult.section != undefined
    ) {
      this.siteStateWriteBuffer = siteStateResult;
      return siteStateResult.section;
    }
    return this.createSection();
  }

  registerClick(): RegisterEventResult {
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
