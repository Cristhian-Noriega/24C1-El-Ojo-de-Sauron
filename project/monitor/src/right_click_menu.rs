#[derive(Clone)]
pub struct RightClickMenu {
    open: bool,
    position: Position,
    pos_2: Pos2,
    id: Id,
    x_coordenate: f64,
    y_coordenate: f64,
}

impl RightClickMenu {
    pub fn default() -> Self {
        Self {
            open: false,
            position: Position::from_lon_lat(0.0, 0.0),
            id: Id::new("right_click_menu"),
            x_coordenate: 0.0,
            y_coordenate: 0.0,
            pos_2: Pos2::new(0.0, 0.0),
        }
    }

    pub fn update(&mut self, click_location_pixels: Pos2, map_response: Response, map_memory: &MapMemory) -> &mut Self {
        let map_center_position = Position::from_lon_lat(DEFAULT_LONGITUDE, DEFAULT_LATITUDE);
    
       // Create a Projector instance
        let projector = Projector::new(map_response.interact_rect, map_memory, map_center_position);

        let mut click_vec2 = click_location_pixels.to_vec2() - map_response.rect.min.to_vec2();

        click_vec2.x = click_vec2.x - map_response.interact_rect.width() / 2.0;
        click_vec2.y = click_vec2.y - map_response.interact_rect.height() / 2.0;

        // Get the geographic coordinates from the click position
        let map_coordinates = projector.unproject(click_vec2);

        println!("Clicked at map coordinates: {:?}", map_coordinates);
        
        self.open = true;
        self.position = map_coordinates;
        self.x_coordenate = map_coordinates.lon();
        self.y_coordenate = map_coordinates.lat();
        self.pos_2 = click_location_pixels;

        self
    }
}