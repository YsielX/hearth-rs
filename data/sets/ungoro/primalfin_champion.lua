local card = {
    api_version = 1, id = "UNG_953", name = "Primalfin Champion",
    text = "[x]<b>Deathrattle:</b> Return any\nspells you cast on this\nminion to your hand.",
    set = "UNGORO", type = "minion", class = "paladin", rarity = "epic",
    cost = 2, attack = 1, health = 3, tags = { "murloc" }, keywords = { "deathrattle" },
}
card.triggers = {{
    event = "spell_targeted", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and event.player_cast and event.target == self
    end,
    effect = function(ctx, self, event)
        local count = ctx:get_data(self, "champion_spell_count") + 1
        ctx:set_data(self, "champion_spell_count", count)
        ctx:set_data(self, "champion_spell_" .. count, event.entity)
    end,
}}
function card.on_deathrattle(ctx, self)
    for index = 1, ctx:get_data(self, "champion_spell_count") do
        local spell = ctx:get_data(self, "champion_spell_" .. index)
        if spell ~= 0 then cardlib.effects.give_card(ctx, ctx:controller(self), ctx:entity(spell).card_id) end
    end
end
return card
