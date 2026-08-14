return {
    api_version = 1,
    id = "UNG_060",
    name = "Mimic Pod",
    text = "Draw a card, then add a copy of it to your hand.",
    set = "UNGORO",
    type = "spell",
    class = "rogue",
    cost = 2,

    on_play = function(ctx, self)
        local player = ctx:controller(self)
        local deck = ctx:deck(player)
        if #deck == 0 then
            ctx:draw(player, 1)
            return
        end
        local card_id = ctx:entity(deck[1]).card_id
        ctx:draw(player, 1)
        ctx:give_card(player, card_id)
    end,
}
