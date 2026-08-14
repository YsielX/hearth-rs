local parts = { "PART_001", "PART_002", "PART_003", "PART_004", "PART_005", "PART_006", "PART_007" }
local card = {
    api_version = 1, id = "GVG_078", name = "Mechanical Yeti",
    text = "<b>Deathrattle:</b> Give each player a <b>Spare Part.</b>", set = "GVG",
    type = "minion", rarity = "common", cost = 4, attack = 4, health = 5,
    tags = { "mech" }, keywords = { "deathrattle" },
}
function card.on_deathrattle(ctx, self)
    ctx:random_value(parts, "give_owner_part")
    ctx:random_value(parts, "give_opponent_part")
end
function card.give_owner_part(ctx, self, part) ctx:give_card(ctx:controller(self), part) end
function card.give_opponent_part(ctx, self, part) ctx:give_card(ctx:opponent(ctx:controller(self)), part) end
return card
