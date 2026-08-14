local card = {
    api_version = 1, id = "CFM_806", name = "Wrathion",
    text = "<b>Taunt</b>. <b>Battlecry:</b> Draw cards until you draw one that isn't a Dragon.",
    set = "GANGS", type = "minion", rarity = "legendary", cost = 6, attack = 4,
    health = 5, keywords = { "taunt", "battlecry" },
}
local function is_dragon(definition)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == "dragon" or tag == "all" then return true end
    end
    return false
end
function card.on_battlecry(ctx, self) ctx:continue_with("draw_next") end
function card.draw_next(ctx, self)
    local player = ctx:controller(self)
    local deck = ctx:deck(player)
    if #deck == 0 then ctx:draw(player, 1) return end
    local entity = deck[1]
    ctx:draw_entity(player, entity)
    ctx:continue_with_entity("check_draw", entity)
end
function card.check_draw(ctx, self, entity)
    if is_dragon(ctx:card_definition(ctx:entity(entity).card_id)) then ctx:continue_with("draw_next") end
end
return card
