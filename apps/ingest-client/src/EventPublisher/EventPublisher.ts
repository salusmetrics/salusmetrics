import { ToPublishEvent } from "./PublishEvent";

export interface EventPublishSuccess {
  count: number;
}

export enum EventPublishError {
  BadRequest,
  InternalServerError,
  Timeout,
}

export type EventPublishResult = EventPublishSuccess | EventPublishError;

export interface EventPublisher {
  publish(events: ToPublishEvent[]): Promise<EventPublishResult>;
}
