local card = {
    api_version = 1, id = "AT_019", name = "Dreadsteed",
    text = "<b>Deathrattle:</b> At the end\n of the turn, summon a Dreadsteed.", set = "TGT",
    type = "minion", class = "warlock", rarity = "epic", cost = 4, attack = 1, health = 1,
    tags = { "demon" }, keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    ctx:set_data(self, "pending_returns", ctx:get_data(self, "pending_returns") + 1)
end

card.triggers = {{
    event = "turn_ended", timing = "after", active_zones = { "graveyard" },
    condition = function(ctx, self) return ctx:get_data(self, "pending_returns") > 0 end,
    effect = function(ctx, self)
        local count = ctx:get_data(self, "pending_returns")
        ctx:set_data(self, "pending_returns", 0)
        for _ = 1, count do ctx:summon(ctx:controller(self), "AT_019") end
    end,
}}

return card
