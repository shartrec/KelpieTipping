-- Table to store teams
CREATE TABLE teams (
    team_id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    nickname VARCHAR(10) NOT NULL
);

-- Table to store rounds
CREATE TABLE rounds (
    round_id SERIAL PRIMARY KEY,
    round_number INT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL
);

-- Table to store matches
CREATE TABLE matches (
    match_id SERIAL PRIMARY KEY,
    round_id INT NOT NULL REFERENCES rounds(round_id),
    home_team_id INT NOT NULL REFERENCES teams(team_id),
    away_team_id INT NOT NULL REFERENCES teams(team_id),
    match_date TIMESTAMP NOT NULL,
    home_team_score INT,
    away_team_score INT
);

-- Table to store users
CREATE TABLE users (
    user_id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE
);

-- Table to store tips
CREATE TABLE tips (
    tip_id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(user_id),
    match_id INT NOT NULL REFERENCES matches(match_id),
    predicted_home_score INT NOT NULL,
    predicted_away_score INT NOT NULL,
    tip_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);