local card = {
    api_version = 1, id = "ICC_201", name = "Roll the Bones",
    text = "Draw a card.\nIf it has <b>Deathrattle</b>, cast this again.",
    set = "ICECROWN", type = "spell", class = "rogue", rarity = "rare", cost = 2,
}

local function has_deathrattle(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords or {}) do
        if keyword == "deathrattle" then return true end
    end
    return false
end

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local deck = ctx:deck(player)
    if #deck == 0 then ctx:draw(player, 1); return end
    local entity = deck[1]
    ctx:draw_entity(player, entity)
    ctx:continue_with_entity("roll_again_if_deathrattle", entity)
end

function card.roll_again_if_deathrattle(ctx, self, entity)
    if ctx:entity(entity).zone == "hand" and has_deathrattle(ctx, entity) then
        ctx:cast_spell(ctx:controller(self), "ICC_201")
    end
end

return card
