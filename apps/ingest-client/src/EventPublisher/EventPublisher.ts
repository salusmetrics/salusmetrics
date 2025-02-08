import { PublishEvent } from "./PublishEvent";

export interface EventPublishSuccess {
  count: number;
}

export enum EventPublishError {
  BadRequest,
  ConfigurationError,
  InternalServerError,
  Timeout,
}

export type EventPublishResult = EventPublishSuccess | EventPublishError;

export interface EventPublisher {
  publish(events: PublishEvent[]): Promise<EventPublishResult>;
}

export interface EventPublisherConfiguration {
  api_key: string;
  host: string;
}

export enum EventPublisherConfigurationState {
  Invalid,
  InvalidApiKey,
  InvalidHost,
  Success,
}

export function validateEventPublisherConfiguration(
  config: EventPublisherConfiguration,
): EventPublisherConfigurationState {
  if (config.api_key.trim().length < 1) {
    return EventPublisherConfigurationState.InvalidApiKey;
  }
  const host = URL.parse(config.host);
  if (host == null) {
    return EventPublisherConfigurationState.InvalidHost;
  }
  if (
    host.pathname.length > 1 ||
    host.search.length > 0 ||
    host.hash.length > 0
  ) {
    return EventPublisherConfigurationState.InvalidHost;
  }
  return EventPublisherConfigurationState.Success;
}
