local parts = { "PART_001", "PART_002", "PART_003", "PART_004", "PART_005", "PART_006", "PART_007" }
local card = {
    api_version = 1, id = "GVG_102", name = "Tinkertown Technician",
    text = "<b>Battlecry:</b> If you have a Mech, gain +1/+1 and add a <b>Spare Part</b> to your hand.",
    set = "GVG", type = "minion", rarity = "common", cost = 3, attack = 3, health = 3,
    keywords = { "battlecry" },
}
local function has_mech(ctx, self)
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self then
            for _, tag in ipairs(ctx:card_definition(ctx:entity(minion).card_id).tags) do
                if tag == "mech" then return true end
            end
        end
    end
    return false
end
function card.on_battlecry(ctx, self)
    if has_mech(ctx, self) then
        cardlib.effects.buff(ctx, self, 1, 1)
        ctx:random_value(parts, "receive_part")
    end
end
function card.receive_part(ctx, self, part) cardlib.effects.give_card(ctx, ctx:controller(self), part) end
return card
