local parts = { "PART_001", "PART_002", "PART_003", "PART_004", "PART_005", "PART_006", "PART_007" }
local card = {
    api_version = 1, id = "GVG_115", name = "Toshley",
    text = "<b>Battlecry and Deathrattle:</b> Add a <b>Spare Part</b> card to your hand.",
    set = "GVG", type = "minion", rarity = "legendary", cost = 6, attack = 5, health = 7,
    keywords = { "battlecry", "deathrattle" },
}
local function part(ctx) ctx:random_value(parts, "receive_part") end
function card.on_battlecry(ctx, self) part(ctx) end
function card.on_deathrattle(ctx, self) part(ctx) end
function card.receive_part(ctx, self, id) ctx:give_card(ctx:controller(self), id) end
return card
