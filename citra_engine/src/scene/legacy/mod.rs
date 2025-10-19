/*
Citra Engine - A Nintendo 3DS first game engine
Copyright (C) 2025  Citra Engine

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

/*
For legacy Scene versions, copy the old Scene struct and deserialize impl here.
Then rename it to SceneV#.
Then implement Into<Scene>
Then add the older version to SceneVersion and increment latest for the new Scene version
*/

mod intermediates;