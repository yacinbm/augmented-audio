// = copyright ====================================================================
// Continuous: Live-looper and performance sampler
// Copyright (C) 2022  Pedro Tacla Yamada
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
// = /copyright ===================================================================
import Combine

public class IntParameter<ParameterId>: ObservableObject, Identifiable {
    public var id: ObjectId
    public var localId: ParameterId
    @Published var label: String
    @Published public var value: Int
    @Published var maximum: Int

    init(id: ObjectId, localId: ParameterId, label: String, value: Int, maximum _: Int) {
        self.id = id
        self.localId = localId
        self.label = label
        self.value = value
        maximum = 0
    }
}