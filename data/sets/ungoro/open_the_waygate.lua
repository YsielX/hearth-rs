local card = { api_version = 1, id = "UNG_028", name = "Open the Waygate",
    text = "[x]<b>Quest:</b> Cast 8 spells that\ndidn't start in your deck.\n<b>Reward:</b> Time Warp.",
    set = "UNGORO", type = "spell", class = "mage", rarity = "legendary", cost = 1,
    keywords = { "quest" }, triggers = {{ event = "spell_cast", timing = "after", active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.player_cast
                and not ctx:entity(event.entity).started_in_deck and ctx:get_data(self, "completed") == 0
        end,
        effect = function(ctx, self)
            local progress = ctx:get_data(self, "progress") + 1
            ctx:set_data(self, "progress", progress)
            if progress >= 8 then
                ctx:set_data(self, "completed", 1); ctx:reveal_secret(self)
                cardlib.effects.give_card(ctx, ctx:controller(self), "UNG_028t")
            end
        end }},
}
card.tokens = {{ id = "UNG_028t", name = "Time Warp", text = "Take an extra turn.\n<i>(Once per game)</i>",
    set = "UNGORO", type = "spell", class = "mage", spell_school = "arcane", cost = 5,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        if ctx:get_player_data(player, "time_warp_used") == 0 then
            ctx:set_player_data(player, "time_warp_used", 1)
            ctx:take_extra_turn(player)
        end
    end }}
return card
