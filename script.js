const GEOSUBMIT_ENDPOINT = "https://api.beacondb.net/v2/geosubmit";
const RUST_SERVER_URL = "http://192.168.0.251:3030/network_json";

let logBuffer = "";

function log(msg) {
  console.log(msg);
  logBuffer += msg + "\n";
}

async function getPosition() {
  try {
    const loc = await Location.current();
    return {
      latitude: loc.latitude,
      longitude: loc.longitude,
      accuracy: loc.horizontalAccuracy,
      altitude: loc.altitude,
      altitudeAccuracy: loc.verticalAccuracy,
      heading: loc.course,        // Heading in degrees
      speed: loc.speed,          // Speed in meters/sec
      source: "gps"
    };
  } catch (e) {
    log("[Location] Failed: " + e.toString());
    return {};
  }
}

// Fetch JSON from Rust server
async function fetchJson() {
  try {
    const req = new Request(RUST_SERVER_URL);
    const json = await req.loadJSON();
    log("[JSON] Fetched items: " + (json.items ? json.items.length : 0));
    return json;
  } catch (e) {
    log("[JSON] Fetch Error: " + e.toString());
    return { items: [] };
  }
}

// Main
async function main() {
  try {
    const data = await fetchJson();
    const position = await getPosition();

    // Overwrite position if GPS available
    if (data.items && data.items.length > 0) {
      data.items[0].position = Object.keys(position).length ? position : data.items[0].position;
    }

    log("[Payload] Ready to submit to BeaconDB:\n" + JSON.stringify(data, null, 2));

    let req; // Define req outside the inner try block to access it in catch
    try {
      req = new Request(GEOSUBMIT_ENDPOINT);
      req.method = "POST";
      req.headers = {
        "Content-Type": "application/json"
      };
      req.body = JSON.stringify(data, null, 2);

      // Use loadString() to get the response without throwing an error
      // if the body is empty or non-JSON (but still a successful status).
      // If the request succeeds (2xx status), loadString() returns
      // the body and the status code is available on req.response.
      await req.loadString();
      
      const statusCode = req.response.statusCode;
      log(`[BeaconDB Response] Status Code: ${statusCode}`);
      
    } catch (e) {
      // If the request fails (4xx or 5xx status), Scriptable throws an
      // error, but the response object is still often attached to the request.
      if (req && req.response && req.response.statusCode) {
        log(`[BeaconDB Submission Error] HTTP Status Code: ${req.response.statusCode}`);
      } else {
        // Handle non-HTTP errors (e.g., network issues)
        log("[BeaconDB Submission Error] " + e.toString());
      }
    }

    QuickLook.present(logBuffer);

  } catch (e) {
    QuickLook.present("[Error] " + e.toString() + "\n\nLogs:\n" + logBuffer);
  }
}

await main();
Script.complete();