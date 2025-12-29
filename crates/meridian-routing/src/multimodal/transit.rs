//! Public transit integration with GTFS support

use crate::error::Result;
use geo_types::Point;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Transit network from GTFS data
pub struct TransitNetwork {
    /// Transit stops
    stops: Vec<TransitStop>,

    /// Transit routes
    routes: Vec<TransitRoute>,

    /// Trips
    trips: Vec<Trip>,

    /// Stop times
    stop_times: HashMap<String, Vec<StopTime>>,
}

impl TransitNetwork {
    /// Load from GTFS directory
    #[cfg(feature = "transit")]
    pub fn from_gtfs<P: AsRef<Path>>(path: P) -> Result<Self> {
        use gtfs_structures::Gtfs;

        let gtfs = Gtfs::from_path(path.as_ref())?;

        let stops = gtfs
            .stops
            .values()
            .map(|s| TransitStop {
                id: s.id.clone(),
                name: s.name.clone(),
                location: Point::new(s.longitude, s.latitude),
                zone_id: s.zone_id.clone(),
            })
            .collect();

        let routes = gtfs
            .routes
            .values()
            .map(|r| TransitRoute {
                id: r.id.clone(),
                short_name: r.short_name.clone(),
                long_name: r.long_name.clone(),
                route_type: map_route_type(r.route_type),
                color: r.color.clone(),
            })
            .collect();

        let trips: Vec<Trip> = gtfs
            .trips
            .values()
            .map(|t| Trip {
                id: t.id.clone(),
                route_id: t.route_id.clone(),
                service_id: t.service_id.clone(),
                headsign: t.trip_headsign.clone(),
            })
            .collect();

        let mut stop_times = HashMap::new();
        for (trip_id, stop_time_vec) in &gtfs.trips {
            let times: Vec<StopTime> = stop_time_vec
                .stop_times
                .iter()
                .map(|st| StopTime {
                    stop_id: st.stop.id.clone(),
                    arrival_time: st.arrival_time.unwrap_or(0),
                    departure_time: st.departure_time.unwrap_or(0),
                    stop_sequence: st.stop_sequence,
                })
                .collect();
            stop_times.insert(trip_id.clone(), times);
        }

        Ok(Self {
            stops,
            routes,
            trips,
            stop_times,
        })
    }

    #[cfg(not(feature = "transit"))]
    pub fn from_gtfs<P: AsRef<Path>>(_path: P) -> Result<Self> {
        Err(crate::error::RoutingError::other(
            "GTFS support not enabled. Enable 'transit' feature.",
        ))
    }

    /// Find nearest stop to location
    pub fn nearest_stop(&self, location: Point) -> Option<&TransitStop> {
        self.stops
            .iter()
            .min_by_key(|stop| {
                let dist = haversine_distance(location, stop.location);
                ordered_float::OrderedFloat(dist)
            })
    }

    /// Find stops within radius (meters)
    pub fn stops_within_radius(&self, location: Point, radius: f64) -> Vec<&TransitStop> {
        self.stops
            .iter()
            .filter(|stop| haversine_distance(location, stop.location) <= radius)
            .collect()
    }

    /// Get departures from stop at time
    pub fn get_departures(
        &self,
        stop_id: &str,
        time: u32,
        max_results: usize,
    ) -> Vec<Departure> {
        let mut departures = Vec::new();

        for (trip_id, times) in &self.stop_times {
            for st in times {
                if st.stop_id == stop_id && st.departure_time >= time {
                    if let Some(trip) = self.trips.iter().find(|t| t.id == *trip_id) {
                        departures.push(Departure {
                            trip_id: trip_id.clone(),
                            route_id: trip.route_id.clone(),
                            headsign: trip.headsign.clone(),
                            departure_time: st.departure_time,
                        });
                    }
                }
            }
        }

        departures.sort_by_key(|d| d.departure_time);
        departures.truncate(max_results);
        departures
    }

    /// Statistics
    pub fn stats(&self) -> TransitStats {
        TransitStats {
            num_stops: self.stops.len(),
            num_routes: self.routes.len(),
            num_trips: self.trips.len(),
        }
    }
}

/// Transit stop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitStop {
    pub id: String,
    pub name: String,
    pub location: Point,
    pub zone_id: Option<String>,
}

/// Transit route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitRoute {
    pub id: String,
    pub short_name: String,
    pub long_name: String,
    pub route_type: RouteType,
    pub color: Option<String>,
}

/// Type of transit route
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RouteType {
    Tram,
    Subway,
    Rail,
    Bus,
    Ferry,
    CableCar,
    Gondola,
    Funicular,
}

/// Trip on a route
#[derive(Debug, Clone)]
struct Trip {
    id: String,
    route_id: String,
    service_id: String,
    headsign: Option<String>,
}

/// Stop time in a trip
#[derive(Debug, Clone)]
struct StopTime {
    stop_id: String,
    arrival_time: u32,
    departure_time: u32,
    stop_sequence: u16,
}

/// Departure information
#[derive(Debug, Clone)]
pub struct Departure {
    pub trip_id: String,
    pub route_id: String,
    pub headsign: Option<String>,
    pub departure_time: u32,
}

/// Transit statistics
#[derive(Debug)]
pub struct TransitStats {
    pub num_stops: usize,
    pub num_routes: usize,
    pub num_trips: usize,
}

#[cfg(feature = "transit")]
fn map_route_type(rt: gtfs_structures::RouteType) -> RouteType {
    match rt {
        gtfs_structures::RouteType::Tramway => RouteType::Tram,
        gtfs_structures::RouteType::Subway => RouteType::Subway,
        gtfs_structures::RouteType::Rail => RouteType::Rail,
        gtfs_structures::RouteType::Bus => RouteType::Bus,
        gtfs_structures::RouteType::Ferry => RouteType::Ferry,
        gtfs_structures::RouteType::CableCar => RouteType::CableCar,
        gtfs_structures::RouteType::Gondola => RouteType::Gondola,
        gtfs_structures::RouteType::Funicular => RouteType::Funicular,
        _ => RouteType::Bus,
    }
}

fn haversine_distance(a: Point, b: Point) -> f64 {
    const EARTH_RADIUS: f64 = 6371000.0;

    let lat1 = a.y().to_radians();
    let lat2 = b.y().to_radians();
    let delta_lat = (b.y() - a.y()).to_radians();
    let delta_lon = (b.x() - a.x()).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS * c
}
