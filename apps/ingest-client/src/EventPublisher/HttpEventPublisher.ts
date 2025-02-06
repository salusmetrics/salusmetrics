import { ToPublishEvent } from "./PublishEvent";
import {
  EventPublisher,
  EventPublishError,
  EventPublishResult,
} from "./EventPublisher";

export class HttpEventPublisher implements EventPublisher {
  private api_key: string;
  private endpoint: string;

  constructor(api_key: string, host: string) {
    this.api_key = api_key;
    this.endpoint = host + "/multi";
  }

  async publish(events: ToPublishEvent[]): Promise<EventPublishResult> {
    const response = await fetch(this.createRequest(events));

    if (response.ok) {
      return { count: events.length };
    }

    if (response.status == 500) {
      return EventPublishError.InternalServerError;
    }

    if (response.status == 400) {
      return EventPublishError.BadRequest;
    }

    return EventPublishError.Timeout;
  }

  private createRequest(events: ToPublishEvent[]): Request {
    return new Request(this.endpoint, {
      method: "POST",
      mode: "cors",
      headers: {
        "content-type": "application/json",
        "api-key": this.api_key,
      },
      priority: "low",
      keepalive: true,
      body: JSON.stringify(events.map((e) => e.toPublishEvent())),
    });
  }
}
