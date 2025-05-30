/*
 * Copyright (c) 2025. Trevor Campbell and others.
 *
 * This file is part of KelpieTipping.
 *
 * KelpieTipping is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License,or
 * (at your option) any later version.
 *
 * KelpieTipping is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with KelpieTipping; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 * Contributors:
 *      Trevor Campbell
 *
 */

-- Table to store teams
CREATE TABLE IF NOT EXISTS teams (
    team_id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    nickname VARCHAR(10) NOT NULL
);

-- Table to store rounds
CREATE TABLE IF NOT EXISTS rounds (
    round_id SERIAL PRIMARY KEY,
    round_number INT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL
);

-- Table to store games
CREATE TABLE IF NOT EXISTS games (
    game_id SERIAL PRIMARY KEY,
    round_id INT NOT NULL REFERENCES rounds(round_id),
    home_team_id INT NOT NULL REFERENCES teams(team_id),
    away_team_id INT NOT NULL REFERENCES teams(team_id),
    game_date DATE NOT NULL,
    home_team_score INT,
    away_team_score INT
);

-- Table to store users
CREATE TABLE IF NOT EXISTS tippers (
    tipper_id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE
);

-- Table to store tips
CREATE TABLE IF NOT EXISTS tips (
    tip_id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES tippers(tipper_id),
    game_id INT NOT NULL REFERENCES games(game_id),
    predicted_home_score INT NOT NULL,
    predicted_away_score INT NOT NULL,
    tip_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);