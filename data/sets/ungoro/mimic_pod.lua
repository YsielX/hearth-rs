local card = {
    api_version = 1,
    id = "UNG_060",
    name = "Mimic Pod",
    text = "Draw a card, then add a copy of it to your hand.",
    set = "UNGORO",
    type = "spell",
    class = "rogue",
    rarity = "rare",
    spell_school = "nature",
    cost = 2,

    on_play = function(ctx, self)
        local player = ctx:controller(self)
        local deck = ctx:deck(player)
        if #deck == 0 then
            ctx:draw(player, 1)
            return
        end
        local entity = deck[1]
        ctx:draw_entity(player, entity)
        ctx:continue_with_entity("copy_mimic_pod_card", entity)
    end,
}

function card.copy_mimic_pod_card(ctx, self, entity)
    if ctx:entity(entity).zone == "hand" then ctx:give_copy(ctx:controller(self), entity) end
end

return card
