local card = {
    api_version = 1, id = "AT_072", name = "Varian Wrynn",
    text = "<b>Battlecry:</b> Draw 3 cards.\nPut any minions you drew directly into the battlefield.",
    set = "TGT", type = "minion", class = "warrior", rarity = "legendary", cost = 10,
    attack = 7, health = 7, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        ctx:set_data(self, "draws_remaining", 3)
        ctx:continue_with("draw_next_card")
    end,
}

function card.draw_next_card(ctx, self)
    local remaining = ctx:get_data(self, "draws_remaining")
    if remaining <= 0 then return end

    local player = ctx:controller(self)
    local deck = ctx:deck(player)
    local expected = deck[1]
    ctx:set_data(self, "draws_remaining", remaining - 1)
    ctx:draw(player, 1)
    ctx:continue_with_value("process_drawn_card", expected or 0)
end

function card.process_drawn_card(ctx, self, entity)
    if entity ~= 0 and ctx:entity(entity).zone == "hand"
        and ctx:entity(entity).type == "minion" then
        ctx:summon_from_hand(entity)
    end
    ctx:continue_with("draw_next_card")
end

return card
