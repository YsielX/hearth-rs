local function is_mech(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "mech" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "GVG_004",
    name = "Goblin Blastmage",
    text = "[x]<b>Battlecry:</b> If you control\na Mech, deal 6 damage\nrandomly split among\nall enemies.",
    set = "GVG",
    type = "minion",
    class = "mage",
    rarity = "rare",
    cost = 4,
    attack = 5,
    health = 4,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self and is_mech(ctx, minion) then
            ctx:set_data(self, "shots_remaining", 6)
            ctx:continue_with("fire_next")
            return
        end
    end
end

function card.fire_next(ctx, self)
    if ctx:get_data(self, "shots_remaining") <= 0 then return end
    local candidates = ctx:enemy_characters(self)
    if #candidates > 0 then ctx:random_entity(candidates, "hit_enemy") end
end

function card.hit_enemy(ctx, self, target)
    ctx:set_data(self, "shots_remaining", ctx:get_data(self, "shots_remaining") - 1)
    cardlib.effects.damage(ctx, target, 1)
    ctx:continue_with("fire_next")
end

return card
