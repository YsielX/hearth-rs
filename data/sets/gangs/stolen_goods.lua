local card = {
    api_version = 1, id = "CFM_752", name = "Stolen Goods",
    text = "Draw a <b>Taunt</b> minion.\nGive it +2/+2.", set = "GANGS",
    type = "spell", class = "warrior", rarity = "rare", cost = 2,
}
local function has_keyword(definition, wanted)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == wanted then return true end
    end
    return false
end
function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local candidates = {}
    for _, entity in ipairs(ctx:deck(player)) do
        if has_keyword(ctx:card_definition(ctx:entity(entity).card_id), "taunt") then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "draw_taunt") end
end
function card.draw_taunt(ctx, self, entity)
    ctx:draw_entity(ctx:controller(self), entity)
    ctx:continue_with_number("buff_drawn", entity)
end
function card.buff_drawn(ctx, self, entity)
    if ctx:entity(entity).zone == "hand" then ctx:buff(entity, 2, 2) end
end
return card
