use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    prelude::SpriteSheetBundle,
    time::FixedTimestep, app::AppExit,};
use rand::prelude::random;

//const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);
//const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
//const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.5,0.5,0.5);
const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

#[derive(PartialEq,Copy,Clone)]
enum Direction{
    Left,
    Up,
    Right,
    Down,
}

impl  Direction {
    fn opposite(self) -> Self{
        match self  {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);


#[derive(Resource, Deref, DerefMut)]
struct DirectionTimer(Timer);

#[derive(Component)]
struct SnakeSegment{
    direction: Direction,
}

#[derive(Resource,Default, Deref, DerefMut)]
struct SnakeSegments(Vec<Entity>);


#[derive(Component)]
struct SnakeHead{
    direction: Direction,
}
#[derive(Resource,Default, Deref, DerefMut)]
struct LastTailPosition(Option<Position>);

#[derive(Resource, Deref, DerefMut)]
struct LastTailDirection(Option<Direction>);

#[derive(Resource, Deref, DerefMut)]
struct HeadDirection(Option<Direction>);

/*impl Default for LastTailDirection {
    fn default() -> Self { LastTailDirection(Some(Direction::Up)) }
}*/
impl Default for HeadDirection {
    fn default() -> Self { HeadDirection(Some(Direction::Up)) }
}

#[derive(Component)]
struct Food;
struct GrowthEvent;

#[derive(Component, Clone, Copy, PartialEq,Eq)]
struct Position{
    x: i32,
    y: i32,
}

struct GameOverEvent;
struct ExitEvent;

#[derive(Component)]
struct Size{
    width:f32,
    height:f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
    
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32 / 24.0,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32 / 24.0,
            1.0,
        );
    }
}
fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}
fn animate_snake_head(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut head_query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
        Option<&SnakeHead>,
        Option<&SnakeSegment>,
    )>,
) {
    for (mut timer, mut sprite,texture_atlas_handle,snakehead,snakesegment) in &mut head_query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();

            if let Some(snakehead) = snakehead {
                match snakehead.direction {
                    Direction::Up => sprite.index=sprite.index% 4,
                    Direction::Down => sprite.index= (sprite.index% 4)+4,
                    Direction::Left => sprite.index=(sprite.index% 4)+8,
                    Direction::Right => sprite.index=(sprite.index% 4)+12,
                   
                    
                }
            }
            if let Some(snakesegment) = snakesegment {
                match snakesegment.direction {
                    Direction::Up => sprite.index=sprite.index% 4,
                    Direction::Down => sprite.index= (sprite.index% 4)+4,
                    Direction::Left => sprite.index=(sprite.index% 4)+8,
                    Direction::Right => sprite.index=(sprite.index% 4)+12,
                   
                    
                }
                  
            }
        }
    }
}
fn spawn_segment(
     mut commands: Commands,
     position: Position,
     asset_server: Res<AssetServer>,
     mut texture_atlases: ResMut<Assets<TextureAtlas>>,
     last_tail_direction: Direction
    ) -> Entity {
    let texture_handle = asset_server.load("norm.png");
    let texture_atlas =
    TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 4, 4, None, None);
    //print!("{}",last_tail_direction.unwrap().display());
    let texture_atlas_handle =  texture_atlases.add(texture_atlas);
    commands
        .spawn((SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                    ..default()
            },
            AnimationTimer(Timer::from_seconds(0.075, TimerMode::Repeating)),
            ))
        .insert(SnakeSegment{
            direction: last_tail_direction,
        })
        .insert(position)
        .insert(Size::square(1.0))
        .id()
}
fn spawn_snake(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    last_tail_direction: Res<LastTailDirection>,
    ){
    let texture_handle = asset_server.load("lead.png");
    let texture_atlas =
    TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 4, 4, None, None);
    let texture_atlas_handle =  texture_atlases.add(texture_atlas);
    *segments = SnakeSegments(vec![
        commands
            .spawn((SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                transform: Transform::from_xyz(
                    3 as f32 * 24.0,
                    3 as f32 * 24.0,
                    5.0,
                ),
                    ..default()
            },
            AnimationTimer(Timer::from_seconds(0.075, TimerMode::Repeating)),
            ))
            .insert(SnakeHead {
                direction: Direction::Up,
            })
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(1.0))
            .id(),
        spawn_segment(commands, Position { x: 3, y: 2 },asset_server,texture_atlases,last_tail_direction.0.unwrap()),
    ]);
}
fn do_not_180(mut commands: Commands) {
    commands.insert_resource(DirectionTimer(Timer::from_seconds(0.3, TimerMode::Once)));
}
fn snake_movement_input(
        time: Res<Time>,
        keyboard_input: Res<Input<KeyCode>>,
        mut direction_timer: ResMut<DirectionTimer>,
        mut heads: Query<&mut SnakeHead>,
        mut head_dir: ResMut<HeadDirection>,){
    
    direction_timer.tick(time.delta());
    if let Some(mut head) = heads.iter_mut().next() {
        if keyboard_input.pressed(KeyCode::Left) {
            *head_dir=HeadDirection(Some(Direction::Left));
        } else if keyboard_input.pressed(KeyCode::Down) {
            *head_dir=HeadDirection(Some(Direction::Down));
        } else if keyboard_input.pressed(KeyCode::Up) {
            *head_dir=HeadDirection(Some(Direction::Up));
        } else if keyboard_input.pressed(KeyCode::Right) {
            *head_dir=HeadDirection(Some(Direction::Right));
        }
        if head_dir.0.unwrap() != head.direction.opposite() &&  direction_timer.just_finished() {
            head.direction = head_dir.0.unwrap();
            direction_timer.reset();
        }
        
        
    }

}

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut last_tail_direction: ResMut<LastTailDirection>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>,
    mut actual_segments: Query<&mut SnakeSegment>,
    
    ) {
   
    {
        if let Some((head_entity, head)) = heads.iter_mut().next() {
            let segment_positions = segments
                .iter()
                .map(|e| *positions.get_mut(*e).unwrap())
                .collect::<Vec<Position>>();
            let mut head_pos = positions.get_mut(head_entity).unwrap();
            match &head.direction {
                Direction::Left => {
                    head_pos.x -= 1;
                }
                Direction::Right => {
                    head_pos.x += 1;
                }
                Direction::Up => {
                    head_pos.y += 1;
                }
                Direction::Down => {
                    head_pos.y -= 1;
                }
            };
            
            if head_pos.x < 0 || head_pos.y < 0 || head_pos.x as u32 >= ARENA_WIDTH || head_pos.y as u32 >= ARENA_HEIGHT{
                game_over_writer.send(GameOverEvent);
            }
            if segment_positions.contains(&head_pos) {
                game_over_writer.send(GameOverEvent);
            }
            let mut ok =1;
            let mut last_direction = head.direction;
            for mut segment in &mut actual_segments {
                if ok ==1{
                    last_direction = segment.direction;
                    segment.direction = head.direction;
                    ok=0;
                }
                else{
                    let temp = segment.direction;
                    segment.direction = last_direction;
                    last_direction = temp;
                }
            }
            *last_tail_direction = LastTailDirection(Some(last_direction));

            segment_positions
                .iter()
                .zip(segments.iter().skip(1))
                .for_each(|(pos, segment)| {
                    *positions.get_mut(*segment).unwrap() = *pos;
                });
            *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));
        }
        }
}
fn get_random_position(
     x: &mut i32,
     y: &mut i32,
    mut positions: Query<&mut Position>,
){
    let mut done =false;
    while done==false
    {
        *x=(random::<f32>() * ARENA_WIDTH as f32) as i32;
        *y=(random::<f32>() * ARENA_HEIGHT as f32) as i32;
        done=true;
        for  pos in &mut positions{
            if pos.x==*x && pos.y==*y {
                done=false;
            }
        }
    }
    
}
fn food_spawner(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    positions: Query<&mut Position>,){
    let texture_handle = asset_server.load("standin.png");
    let texture_atlas =
    TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 3, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let  a: &mut i32 = &mut 0;
    let  b: &mut i32 = &mut 0;
    get_random_position(a,b,positions);
    commands
        .spawn((SpriteSheetBundle{
                texture_atlas: texture_atlas_handle.clone(),
                ..default()
            },
            AnimationTimer(Timer::from_seconds(0.075, TimerMode::Repeating)),
            ))
        .insert(Food)
        .insert(Position{
            x: *a,
            y: *b,
        })
        .insert(Size::square(1.0));
}
fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(ent).despawn();
                growth_writer.send(GrowthEvent);
            }
        }
    }
}
fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    last_tail_direction: Res<LastTailDirection>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if growth_reader.iter().next().is_some() {
        segments.push(spawn_segment(commands, last_tail_position.0.unwrap(),asset_server,texture_atlases,last_tail_direction.0.unwrap()));
    }
}
fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
    mut exit_writer: EventWriter<ExitEvent>,)
    {
    if reader.iter().next().is_some() {
        for ent in food.iter().chain(segments.iter()) {
            commands.entity(ent).despawn();
        }
        exit_writer.send(ExitEvent);
    }
}
fn exit(
    mut reader: EventReader<ExitEvent>,
    mut exit: EventWriter<AppExit>,)
    {
    if reader.iter().next().is_some() {
        exit.send(AppExit);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
fn play_music(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play_with_settings(
        asset_server.load("deepest_pit_of_hell.ogg"),
        PlaybackSettings::LOOP.with_volume(0.75),
    );
}
fn main() {
    App::new()
        //observe that we don't give any input to the functions, bevy weirdly figures it out\
        
        .insert_resource(ClearColor(Color::rgb(255.0, 255.0, 255.0)))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .insert_resource(LastTailDirection(Some(Direction::Up)))
        .insert_resource(HeadDirection::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_snake)
        .add_startup_system(do_not_180)
        //.add_startup_system(play_music)
        .add_system(snake_movement_input.after(do_not_180))
        .add_system(animate_snake_head)
        .add_system(game_over.after(snake_movement))
        .add_system(exit.after(game_over))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
            .with_system(position_translation)
            .with_system(size_scaling),
        )
        
        .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.3))
                    .with_system(snake_movement.after(snake_movement_input))
                    .with_system(snake_eating.after(snake_movement))
                    .with_system(snake_growth.after(snake_eating))
        )
        
        .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(2.0))
                    .with_system(food_spawner)
        )

        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .add_event::<ExitEvent>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Cookie Rampage".to_string(),
                width: 700.0,
                height: 700.0,
                ..default()
            },
            ..default()
            },
        )
        .set(ImagePlugin::default_nearest()))
        .run();
}