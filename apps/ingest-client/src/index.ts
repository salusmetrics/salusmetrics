import {v7 as uuidv7} from 'uuid';

function generateId() {
  let id = uuidv7();
  console.log(id);
}

generateId();
