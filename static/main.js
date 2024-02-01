
const drawBoundingBox = (list_box) => {
  const colors = [
    "red",
    "green",
    "white"
  ];
  const labels = [
    "old",
    "ready",
    "young",
  ];

  let videoOverlay = document.getElementById('video-overlay');
  videoOverlay.innerHTML = '';

  for (let i = 0; i < list_box.length; i++) {
    const pos = list_box[i];

    let box = document.querySelector('#bbox-entry').content.cloneNode(true);
    template.querySelector('p').innerText = labels[pos.object_id];

    box.style.position = "absolute";
    box.style.left = pos.x + "px";
    box.style.top = pos.y + "px";
    box.style.width = pos.w + "px";
    box.style.height = pos.h + "px";
    box.style.border = "5px solid " + colors[pos.object_id];

    videoOverlay.appendChild(box);
  }
}

let promptQueue = [];

const promptShow = () => {
  let modal = document.getElementById('prompt');
  if (modal.classList.contains('is-active')) {
    return;
  }

  let modalBody = document.getElementById('prompt-body');
  modalBody.innerHTML = '';

  const promptEntry = promptQueue.pop();
  const entry = Object.keys(promptEntry)[0];
  const entries = promptEntry[entry];
  const keys = Object.keys(entries);

  document.getElementById('prompt-name').value = entry;
  document.getElementById('prompt-title').innerText = entry;

  for (let i = 0; i < keys.length; i++) {
    const name = keys[i];
    const value = entries[name];

    console.log(name, value)

    if (typeof value === 'boolean') {
      let template = document.querySelector('#bool-prompt-entry').content.cloneNode(true);
      template.querySelector('label').innerText = name;
      let input = template.querySelector('input');
      input.name = name;
      input.checked = value === true;
      modalBody.appendChild(template);
    } else if (typeof value === 'number') {
      let template = document.querySelector('#number-prompt-entry').content.cloneNode(true);
      template.querySelector('label').innerText = name;
      let input = template.querySelector('input');
      input.name = name;
      input.value = value;
      modalBody.appendChild(template);
    } else if (typeof value === 'string') {
      let template = document.querySelector('#text-prompt-entry').content.cloneNode(true);
      template.querySelector('label').innerText = name;
      let input = template.querySelector('input');
      input.name = name;
      input.value = value;
      modalBody.appendChild(template);
    }
  }
  document.getElementById('prompt').classList.add('is-active');
}

const promptSubmit = async () => {
  let modalBody = document.getElementById('prompt-body');

  const promptName = document.getElementById('prompt-name').value;
  let obj = {};
  obj.Setup = {};
  obj.Setup[promptName] = {};

  const entries = modalBody.querySelectorAll('input');

  for (let i = 0; i < entries.length; i++) {
    let key = entries[i].name;
    let value;

    if (entries[i].type == 'number') {
      value = parseInt(entries[i].value);
    } else if (entries[i].type == 'checkbox') {
      value = entries[i].checked == true;
    } else {
      value = entries[i].value;
    }

    obj.Setup[promptName][key] = value;
  }
  console.log(obj);
  await fetch('/push', {
    method: 'POST',
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(obj),
  });
  if (promptQueue.length > 0) {
    promptShow();
  }
}
new EventSource("/pull").onmessage = (event) => {
  //let videoScreen = document.getElementById('video-screen');
  let statusBarText = document.getElementById('status-bar-text');

  let msg = JSON.parse(event.data);
  if (msg.Error != null) {
    statusBarText.style.color = 'red';
    statusBarText.innerText = msg.Error;
    console.error(msg.Error);
  } else if (msg.Status != null) {
    statusBarText.style.color = 'green';
    statusBarText.innerText = msg.Status;
    console.log(msg.Status);
  } else if (msg.ReportListBox != null) {
    drawBoundingBox(msg.ReportListBox);
  } else if (msg.Prompt != null) {
    promptQueue.push(msg.Prompt);
    promptShow();

    console.log(msg.Prompt);
  } else {
    console.log(msg);
  }
};

const hello = async () => {
  await fetch('/push', {
    method: 'POST',
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify("Hello"),
  });
}

hello();
