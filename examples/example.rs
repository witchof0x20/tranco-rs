// Copyright 2024 witchof0x20
//
// This file is part of tranco-rs.
//
// tranco-rs is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// tranco-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with tranco-rs. If not, see <https://www.gnu.org/licenses/>.
use tranco::Client;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let ranks = client.ranks("google.com").await;
    dbg!(&ranks);
    let list = client.list("LJL44").await;
    dbg!(&list);
    let list2 = client.list_date(2025, 04, 07, Some(false)).await;
    dbg!(&list2);
    let downloaded_list = client.download_list(&list2.unwrap()).await;
    dbg!(&downloaded_list);
}
