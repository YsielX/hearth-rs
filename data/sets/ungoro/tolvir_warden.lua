local card = { api_version = 1, id = "UNG_913", name = "Tol'vir Warden",
    text = "<b>Battlecry:</b> Draw two 1-Cost minions from your deck.", set = "UNGORO",
    type = "minion", class = "hunter", rarity = "rare", cost = 4, attack = 3, health = 4,
    keywords = { "battlecry" } }
function card.on_battlecry(ctx, self) ctx:set_data(self, "draws_remaining", 2); ctx:continue_with("draw_next") end
function card.draw_next(ctx, self)
    if ctx:get_data(self, "draws_remaining") <= 0 then return end
    local candidates = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        local card_entity = ctx:entity(entity)
        if card_entity.type == "minion" and card_entity.cost == 1 then candidates[#candidates + 1] = entity end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "draw_selected") end
end
function card.draw_selected(ctx, self, entity)
    ctx:set_data(self, "draws_remaining", ctx:get_data(self, "draws_remaining") - 1)
    ctx:draw_entity(ctx:controller(self), entity)
    ctx:continue_with("draw_next")
end
return card
