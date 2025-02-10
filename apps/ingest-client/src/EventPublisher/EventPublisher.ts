import { PublishEvent } from "./PublishEvent";

export interface EventPublishSuccess {
  eventCount: number;
}

export enum EventPublishError {
  BadRequest,
  ConfigurationError,
  FetchError,
  InternalServerError,
  Timeout,
}

export type EventPublishResult = EventPublishSuccess | EventPublishError;

export function isEventPublishResultError(
  result: EventPublishResult,
): result is EventPublishError {
  return typeof result == "number";
}

export interface EventPublisher {
  publish(events: PublishEvent[]): Promise<EventPublishResult>;
}
