import {v7 as uuidv7} from 'uuid';

const enum EventType {
  Visitor = 1,
  Session = 2,
  Section = 3,
  Click = 4
}

interface Event {
  event_type: EventType,
  id: string,
  attrs: Record<string, string> | undefined
}

function createEvent(event_type: EventType, attrs?: Record<string, string>): Event {
  let event = { event_type, attrs, id: uuidv7() };
  console.log(event);
  return event;
}

let visitor_event = createEvent(EventType.Visitor);
let sessionRecord: Record<string, string> = {};
sessionRecord["parent"] = visitor_event.id;
let session_event = createEvent(EventType.Session, sessionRecord);

console.log(sessionRecord);

let body = JSON.stringify([visitor_event, session_event]);

fetch("http://localhost:3000/multi", {
  "headers": {
      "Content-Type": "application/json",
      "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:133.0) Gecko/20100101 Firefox/133.0",
      "api-key": "abc-xyz",
      "Accept-Language": "en-US,en;q=0.5"
  },
  "body": body,
  "method": "POST",
  "mode": "cors"
});
