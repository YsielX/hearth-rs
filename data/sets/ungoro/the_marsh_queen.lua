local card = { api_version = 1, id = "UNG_920", name = "The Marsh Queen",
    text = "[x]<b>Quest:</b> Play seven\n1-Cost minions.\n<b>Reward:</b> Queen Carnassa.",
    set = "UNGORO", type = "spell", class = "hunter", rarity = "legendary", cost = 1,
    keywords = { "quest" }, triggers = {{ event = "card_played", timing = "after", active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and ctx:get_data(self, "completed") == 0
                and ctx:entity(event.entity).type == "minion" and event.cost == 1
        end,
        effect = function(ctx, self)
            local progress = ctx:get_data(self, "progress") + 1
            ctx:set_data(self, "progress", progress)
            if progress >= 7 then
                ctx:set_data(self, "completed", 1); ctx:reveal_secret(self)
                cardlib.effects.give_card(ctx, ctx:controller(self), "UNG_920t1")
            end
        end }},
}
card.tokens = {
    { id = "UNG_920t1", name = "Queen Carnassa", text = "<b>Rush</b>\n<b>Battlecry:</b> Shuffle 20 Raptors into your deck.",
      set = "UNGORO", type = "minion", class = "hunter", cost = 5, attack = 8, health = 8,
      tags = { "beast" }, keywords = { "rush", "battlecry" },
      on_battlecry = function(ctx, self) for _ = 1, 20 do cardlib.effects.shuffle_card_into_deck(ctx, ctx:controller(self), "UNG_920t2") end end },
    { id = "UNG_920t2", name = "Carnassa's Brood", text = "<b>Battlecry:</b> Draw a card.",
      set = "UNGORO", type = "minion", class = "hunter", cost = 1, attack = 3, health = 2,
      tags = { "beast" }, keywords = { "battlecry" }, on_battlecry = function(ctx, self) ctx:draw(ctx:controller(self), 1) end },
}
return card
