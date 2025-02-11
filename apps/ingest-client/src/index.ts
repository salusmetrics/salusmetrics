import { EventConfiguration } from "common/Event/EventConfiguration";
import { EventManager } from "main-thread/EventManager/EventManager";
import { WorkerPublisherGateway } from "main-thread/EventPublisher/WorkerPublisherGateway";
import { WebStorageSiteStateRepository } from "main-thread/SiteState/WebStorageSiteStateRepository";

const config: EventConfiguration = {
  api_key: "abc-xyz",
  host: "http://localhost:3000",
};

let siteStateRepository = new WebStorageSiteStateRepository(config);
let gateway = new WorkerPublisherGateway(config, (e) => console.log(e));
let eventManager = new EventManager(gateway, siteStateRepository);

eventManager.registerSection();
