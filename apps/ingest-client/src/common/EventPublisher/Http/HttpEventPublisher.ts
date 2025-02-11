import { PublishEvent } from "../PublishEvent";
import {
  EventPublisher,
  EventPublishError,
  EventPublishResult,
} from "../EventPublisher";
import {
  EventConfiguration,
  EventConfigurationState,
  validateEventConfiguration,
} from "../../Event/EventConfiguration";

export class HttpEventPublisher implements EventPublisher {
  private api_key: string;
  private endpoint: string;

  constructor(config: EventConfiguration) {
    if (validateEventConfiguration(config) != EventConfigurationState.Success) {
      throw new Error("Invalid Event Publisher Configuration");
    }
    this.api_key = config.api_key;
    this.endpoint = config.host + "/multi";
  }

  async publish(events: PublishEvent[]): Promise<EventPublishResult> {
    try {
      const response = await fetch(this.createRequest(events));

      if (response.ok) {
        return { eventCount: events.length };
      }

      if (response.status == 500) {
        return EventPublishError.InternalServerError;
      }

      if (response.status == 400) {
        return EventPublishError.BadRequest;
      }

      return EventPublishError.Timeout;
    } catch (e) {
      return EventPublishError.FetchError;
    }
  }

  private createRequest(events: PublishEvent[]): Request {
    return new Request(this.endpoint, {
      method: "POST",
      mode: "cors",
      headers: {
        "content-type": "application/json",
        "api-key": this.api_key,
      },
      keepalive: true,
      body: JSON.stringify(events),
    });
  }
}
