import { Visitor } from "./Event/Event";
// import { EventManager } from "./EventManager/EventManager";
// import { HttpEventPublisher } from "./EventPublisher/HttpEventPublisher";
import { WorkerPublisherGateway } from "./EventPublisher/WorkerPublisherGateway";
// import { WebStorageSiteStateRepository } from "./SiteState/WebStorageSiteStateRepository";

let api_key = "abc-xyz";
let host = "http://localhost:3000";

// let siteStateRepository = new WebStorageSiteStateRepository(api_key);
// let publisher = new HttpEventPublisher({
//   api_key,
//   host: "http://localhost:3000",
// });
// let eventManager = new EventManager(publisher, siteStateRepository);

// eventManager.registerSection();

let visitor: Visitor = new Visitor();

let gateway = new WorkerPublisherGateway({ api_key, host }, (e) =>
  console.log(e),
);

gateway.publish([visitor.toPublishEvent()]);
