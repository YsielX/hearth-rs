return {
    api_version = 1, id = "CFM_669", name = "Burgly Bully",
    text = "Whenever your opponent casts a spell, add a Coin to your hand.",
    set = "GANGS", type = "minion", rarity = "epic", cost = 5, attack = 4, health = 6,
    triggers = {{
        event = "spell_cast", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player_cast and event.player == ctx:opponent(ctx:controller(self))
        end,
        effect = function(ctx, self) cardlib.effects.give_card(ctx, ctx:controller(self), "GAME_005") end,
    }},
}
