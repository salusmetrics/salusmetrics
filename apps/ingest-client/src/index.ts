// import { Visitor } from "./Event/Event";
import { EventConfiguration } from "./Event/EventConfiguration";
import { EventManager } from "./EventManager/EventManager";
// import { HttpEventPublisher } from "./EventPublisher/HttpEventPublisher";
import { WorkerPublisherGateway } from "./EventPublisher/WorkerPublisherGateway";
import { WebStorageSiteStateRepository } from "./SiteState/WebStorageSiteStateRepository";

const config: EventConfiguration = {
  api_key: "abc-xyz",
  host: "http://localhost:3000",
};

let siteStateRepository = new WebStorageSiteStateRepository(config);
let gateway = new WorkerPublisherGateway(config, (e) => console.log(e));
let eventManager = new EventManager(gateway, siteStateRepository);

eventManager.registerSection();
