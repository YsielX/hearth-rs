local card = {
    api_version = 1, id = "GVG_075", name = "Ship's Cannon",
    text = "[x]After you summon a\nPirate, deal 2 damage\nto a random enemy.", set = "GVG",
    type = "minion", rarity = "common", cost = 2, attack = 2, health = 3,
}
local function is_pirate(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "pirate" then return true end
    end
    return false
end
card.triggers = {{
    event = "minion_summoned", active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and is_pirate(ctx, event.entity)
    end,
    effect = function(ctx, self) ctx:random_entity(ctx:enemy_characters(self), "fire") end,
}}
function card.fire(ctx, self, target) ctx:damage(target, 2) end
return card
