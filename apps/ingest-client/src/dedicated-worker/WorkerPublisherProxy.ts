import { EventConfigurationState } from "common/Event/EventConfiguration";
import {
  EventPublishError,
  EventPublishResult,
} from "common/EventPublisher/EventPublisher";
import { HttpEventPublisher } from "common/EventPublisher/Http/HttpEventPublisher";
import {
  WorkerConfigurationRequestMessage,
  WorkerConfigurationResultMessage,
  WorkerPublishErrorMessage,
  WorkerMessage,
  WorkerMessageType,
  WorkerPublishRequestMessage,
  WorkerPublishResultMessage,
} from "common/EventPublisher/Worker/WorkerMessage";

export class WorkerPublisherProxy {
  private httpPublisher: HttpEventPublisher | undefined = undefined;
  private workerContext: DedicatedWorkerGlobalScope;

  constructor(workerContext: DedicatedWorkerGlobalScope) {
    this.workerContext = workerContext;
  }

  private configurePublisher(
    configMessage: WorkerConfigurationRequestMessage,
  ): void {
    try {
      this.httpPublisher = new HttpEventPublisher(configMessage.config);
    } catch (err) {
      const errMessage: WorkerConfigurationResultMessage = {
        configurationState: EventConfigurationState.Invalid,
        id: configMessage.id,
        messageType: WorkerMessageType.ConfigureResult,
      };
      this.postResponse(errMessage);
    }
  }

  private publishEvents(publishMessage: WorkerPublishRequestMessage) {
    if (this.httpPublisher == undefined) {
      const errorMessage: WorkerPublishErrorMessage = {
        id: publishMessage.id,
        errorType: EventPublishError.ConfigurationError,
        errorMessage: "Publish event request received prior to configuration",
        messageType: WorkerMessageType.PublishError,
      };
      this.postResponse(errorMessage);
      return;
    }
    if (publishMessage.events.length < 1) {
      const errorMessage: WorkerPublishErrorMessage = {
        id: publishMessage.id,
        errorType: EventPublishError.BadRequest,
        errorMessage: "Request to publish empty set of events",
        messageType: WorkerMessageType.PublishError,
      };
      this.postResponse(errorMessage);
      return;
    }
    this.httpPublisher
      .publish(publishMessage.events)
      .then((result) => this.publishResult(publishMessage.id, result));
  }

  private publishResult(messageId: string, result: EventPublishResult) {
    const resultMessage: WorkerPublishResultMessage = {
      id: messageId,
      messageType: WorkerMessageType.PublishResult,
      result,
    };
    this.postResponse(resultMessage);
  }

  private postResponse(message: WorkerMessage) {
    this.workerContext.postMessage(message);
  }

  proxyMessage(message: MessageEvent) {
    const workerMessage = message.data as
      | WorkerConfigurationRequestMessage
      | WorkerPublishRequestMessage;

    if (workerMessage.messageType == WorkerMessageType.ConfigureRequest) {
      this.configurePublisher(workerMessage);
    } else if (workerMessage.messageType == WorkerMessageType.PublishRequest) {
      this.publishEvents(workerMessage);
    }
  }
}
