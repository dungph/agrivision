const timeDistanceFromNow = (inputDate) => {
  const now = new Date();
  const diffInSeconds = (now - new Date(inputDate)) / 1000;
  const diffInMinutes = diffInSeconds / 60;
  const diffInHours = diffInMinutes / 60;
  const diffInDays = diffInHours / 24;
  const diffInMonths = diffInDays / 30;
  const diffInYears = diffInMonths / 12;

  if (diffInSeconds < 30) {
    return "less than a minute";
  } else if (diffInSeconds < 90) {
    return "1 minute";
  } else if (diffInMinutes < 44.5) {
    return `${Math.round(diffInMinutes)} minutes`;
  } else if (diffInMinutes < 89.5) {
    return "about 1 hour";
  } else if (diffInHours < 23.98) {
    return `about ${Math.round(diffInHours)} hours`;
  } else if (diffInHours < 41.98) {
    return "1 day";
  } else if (diffInDays < 30) {
    return `${Math.round(diffInDays)} days`;
  } else if (diffInDays < 45) {
    return "about 1 month";
  } else if (diffInDays < 60) {
    return "about 2 months";
  } else if (diffInMonths < 12) {
    return `${Math.round(diffInMonths)} months`;
  } else if (diffInMonths < 15) {
    return "about 1 year";
  } else if (diffInMonths < 21) {
    return "over 1 year";
  } else if (diffInMonths < 24) {
    return "almost 2 years";
  } else {
    const roundedYears = Math.floor(diffInYears);
    const remainingMonths = diffInMonths - roundedYears * 12;

    if (remainingMonths < 3) {
      return `about ${roundedYears} years`;
    } else if (remainingMonths < 9) {
      return `over ${roundedYears} years`;
    } else {
      return `almost ${roundedYears + 1} years`;
    }
  }
}

const convertTimestamp = () => {
  document.querySelectorAll(".convert-timestamp").forEach((el) => {
    let dt = new Date(1000 * el.getAttribute("timestamp")).toLocaleString();
    el.innerHTML = '' + dt + " (" + timeDistanceFromNow(dt) + " ago)"
  });
}
window.onload = () => {
  const updateImages = () => {
    document.querySelectorAll("img.reload").forEach((el) => {
      if (el.complete) {
        el.src = el.src.split("?")[0] + "?" + new Date().getTime();
      }
    });
  }

  setInterval(updateImages, 200);

  convertTimestamp();
}

const reload_elements = () => {
  document.querySelectorAll(".filter-stage-button.is-normal").forEach((el) => {
    el.onclick = () => {
      el.classList.remove("is-normal");
      el.classList.add("is-active");

      document.querySelectorAll(".filter-stage-button.is-normal").forEach((el) => {
        el.classList.remove("is-normal");
        el.classList.add("is-inactive");
      });

      let current_stage = el.getAttribute("stage");
      document.querySelectorAll(".position-cell").forEach((el) => {
        if (current_stage !== el.getAttribute("stage")) {
          el.classList.remove("is-normal");
          el.classList.add("is-inactive");
        }
      });

      reload_elements();
    }
  });
  document.querySelectorAll(".filter-stage-button.is-active").forEach((el) => {
    el.onclick = () => {
      el.classList.add("is-normal");
      el.classList.remove("is-active");

      document.querySelectorAll(".is-inactive").forEach((el) => {
        el.classList.add("is-normal");
        el.classList.remove("is-inactive");
      });
      reload_elements();
    }
  });
  document.querySelectorAll(".expand-button").forEach((el) => {
    el.onclick = () => {
      document.body.classList.add("expanded");
      document.querySelectorAll(".expand-button").forEach((el) => {
        el.classList.remove("expand-button");
        el.classList.add("collapse-button");
      });
      localStorage.setItem("expanded", true);
      reload_elements();
    }
  })
  document.querySelectorAll(".collapse-button").forEach((el) => {
    el.onclick = () => {
      document.body.classList.remove("expanded");
      document.querySelectorAll(".collapse-button").forEach((el) => {
        el.classList.remove("collapse-button");
        el.classList.add("expand-button");
      });
      localStorage.removeItem("expanded");
      reload_elements();
    }
  })
  if (localStorage.getItem("expanded") === null) {
    document.querySelectorAll(".collapse-button").forEach((el) => el.click());
  } else {
    document.querySelectorAll(".expand-button").forEach((el) => el.click());
  }
}
reload_elements();

const gotoPosition = async (x, y) => {
  document.querySelector("#loading").classList.add("reload");
  document.querySelector("#camera-stream").style.display = '';
  await fetch("/action/goto?x=" + x + "&y=" + y);
  document.querySelector("#loading").classList.remove("reload");
  document.querySelector("#loading").style.display = 'none';
}

const recheck = async (id) => {
  await fetch("/action/recheck?id=" + id);
}

const checkPosition = async (id) => {
  document.querySelector("#loading").style.display = '';
  let image = document.querySelector("#current-card-image");
  image.src = "/camera/snapshot";
  image.classList.add("reload");
  await fetch("/action/check?id=" + id);
  document.querySelector("#loading").style.display = 'none';
  window.location.reload();
}

const waterPosition = async (id) => {
  document.querySelector("#loading").style.display = '';
  let image = document.querySelector("#current-card-image");
  image.src = "/camera/snapshot";
  image.classList.add("reload");
  await fetch("/action/water?id=" + id);
  document.querySelector("#loading").style.display = 'none';
  window.location.reload();
}
const checkAll = async () => {
  document.querySelector("#loading").style.display = '';

  let list = document.querySelectorAll(".position-cell.is-normal");
  for (let i = 0; i < list.length; i++) {
    let id = list.item(i).getAttribute("pos-id");
    await fetch("/action/check?id=" + id);
  }
  document.querySelector("#loading").style.display = 'none';
  window.location.reload();
}

const waterAll = async () => {
  document.querySelector("#loading").style.display = '';
  let list = document.querySelectorAll(".position-cell.is-normal");
  for (let i = 0; i < list.length; i++) {
    let id = list.item(i).getAttribute("pos-id");
    await fetch("/action/water?id=" + id);
  }
  document.querySelector("#loading").style.display = 'none';
  window.location.reload();
}

let timeout;

const get_ts = (el) => parseInt(el.getAttribute('timestamp'));
const get_image_id = (el) => parseInt(el.getAttribute('image_id'));
const get_check_id = (el) => parseInt(el.getAttribute('check_id'));

const elements = Array.from(document.querySelectorAll('.history-item'));

let current_element = document.querySelector('.history-item');

const apply_history = (el) => {
  let ts = get_ts(el);
  let image_id = get_image_id(el);
  let check_id = get_check_id(el);
  document.querySelectorAll(".check-image").forEach((el) => {
    el.src = "/camera/image?id=" + image_id;
  });
  document.querySelectorAll(".recheck-button").forEach(el => {
    el.setAttribute("onclick", "recheck(" + check_id + ")")
  })

  document.querySelector(".current-check-ts").setAttribute('timestamp', get_ts(el));

  current_element = elements.find(el => get_ts(el) === ts);
  convertTimestamp();
}

const begin = () => {
  current_element = elements.reduce((a, b) => get_ts(a) < get_ts(b) ? a : b);
  apply_history(current_element);
}

const end = () => {
  current_element = elements.reduce((a, b) => get_ts(a) > get_ts(b) ? a : b);
  apply_history(current_element);
}

const play = () => {
  const iterate = () => {
    if (!current_element) return;

    apply_history(current_element);

    let currentTimestamp = get_ts(current_element);
    console.log(currentTimestamp);

    // Find the next element with a timestamp greater than the current_element's timestamp
    let next_element = elements
      .filter(el => get_ts(el) > get_ts(current_element))
      .reduce((a, b) => get_ts(a) < get_ts(b) ? a : b);

    if (next_element) {
      let nextTimestamp = get_ts(next_element);
      let delay = nextTimestamp - currentTimestamp;

      // Set current_element to the next element
      current_element = next_element;

      // Convert timestamp difference to milliseconds for setTimeout
      timeout = setTimeout(iterate, 1000);
    } else {
      clearTimeout(timeout);
    }
  }

  iterate(); // Start the iteration
}

function pause() {
  clearTimeout(timeout);
}
