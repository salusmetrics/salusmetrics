import { EventType } from "../Event/Event";

export interface PublishEvent {
  t: EventType;
  i: string;
  a: Record<string, string> | undefined;
}

export interface ToPublishEvent {
  toPublishEvent(): PublishEvent;
}
