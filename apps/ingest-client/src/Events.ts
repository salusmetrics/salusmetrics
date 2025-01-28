import { ClientEvent, EventType } from "./ClientEvent";
import {v7 as uuidv7} from 'uuid';

export interface EventReference {
  readonly event_type: EventType;
  readonly id: string;
}

export interface Event extends EventReference {
  toClientEvent(): ClientEvent;
}

export interface VisitorEvent extends Event {
  event_type: EventType.Visitor;
}

export interface VisitorReference extends EventReference {
  event_type: EventType.Visitor;
}

export interface SessionEvent extends Event {
  parent: VisitorReference;
}

export interface SessionReference extends EventReference {
  event_type: EventType.Session;
}

export interface SectionEvent extends Event {
  event_type: EventType.Section;
  parent: SessionReference;
}

export interface SectionReference extends EventReference {
  event_type: EventType.Section;
}

export interface ClickEvent extends Event {
  event_type: EventType.Click;
  parent: SectionReference;
}

export class Visitor implements VisitorEvent, VisitorReference {
  readonly event_type: EventType.Visitor;
  readonly id: string;

  constructor() {
    this.event_type = EventType.Visitor;
    this.id = uuidv7();
  }
  
  toClientEvent(): ClientEvent {
    return {
      t: this.event_type,
      i: this.id,
      a: undefined,
    };
  }
}

export class Session implements SessionEvent, SessionReference {
  readonly event_type: EventType.Session;
  readonly id: string;
  readonly parent: VisitorReference;

  constructor(parent: VisitorReference) {
    this.event_type = EventType.Session;
    this.id = uuidv7();
    this.parent = parent;
  }
  
  toClientEvent(): ClientEvent {
    const attrs: Record<string, string> = { parent: this.parent.id };
    return {
      t: this.event_type,
      i: this.id,
      a: attrs,
    };
  }
}

export class Section implements SectionEvent, SectionReference {
  readonly event_type: EventType.Section;
  readonly id: string;
  readonly parent: SessionReference;

  constructor(parent: SessionReference) {
    this.event_type = EventType.Section;
    this.id = uuidv7();
    this.parent = parent;
  }
  
  toClientEvent(): ClientEvent {
    const attrs: Record<string, string> = { parent: this.parent.id };
    return {
      t: this.event_type,
      i: this.id,
      a: attrs,
    };
  }
}

export class Click implements ClickEvent {
  readonly event_type: EventType.Click;
  readonly id: string;
  readonly parent: SectionReference;

  constructor(parent: SectionReference) {
    this.event_type = EventType.Click;
    this.id = uuidv7();
    this.parent = parent;
  }
  
  toClientEvent(): ClientEvent {
    const attrs: Record<string, string> = { parent: this.parent.id };
    return {
      t: this.event_type,
      i: this.id,
      a: attrs,
    };
  }
}
