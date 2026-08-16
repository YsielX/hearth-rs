local card = {
    api_version = 1, id = "UNG_113", name = "Bright-Eyed Scout",
    text = "<b>Battlecry:</b> Draw a card. Change its Cost to (5).",
    set = "UNGORO", type = "minion", rarity = "epic", cost = 3, attack = 3, health = 4,
    keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local deck = ctx:deck(ctx:controller(self))
    if #deck == 0 then ctx:draw(ctx:controller(self), 1) return end
    ctx:draw_entity(ctx:controller(self), deck[1])
    ctx:continue_with_entity("set_scout_card_cost", deck[1])
end
function card.set_scout_card_cost(ctx, self, entity)
    if ctx:entity(entity).zone == "hand" then
        cardlib.effects.modify(ctx, entity, { stat = "cost", operation = "final_set", value = 5 })
    end
end
return card
