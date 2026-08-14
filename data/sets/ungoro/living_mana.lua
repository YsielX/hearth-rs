local card = { api_version = 1, id = "UNG_111", name = "Living Mana",
    text = "Transform your Mana Crystals into 2/2 Treants. Recover the mana when they die.",
    set = "UNGORO", type = "spell", class = "druid", rarity = "epic", spell_school = "nature", cost = 5 }
function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local count = math.min(ctx:player(player).max_mana, 7 - #ctx:board(player))
    if count <= 0 then return end
    ctx:destroy_mana_crystals(player, count)
    ctx:set_data(self, "treants_remaining", count)
    ctx:continue_with("summon_next_treant")
end
function card.summon_next_treant(ctx, self)
    local remaining = ctx:get_data(self, "treants_remaining")
    if remaining <= 0 then return end
    ctx:set_data(self, "treants_remaining", remaining - 1)
    ctx:summon(ctx:controller(self), "UNG_111t1")
    ctx:continue_with("summon_next_treant")
end
card.tokens = {{ id = "UNG_111t1", name = "Mana Treant", text = "<b>Deathrattle:</b> Gain an empty Mana Crystal.",
    set = "UNGORO", type = "minion", class = "druid", cost = 1, attack = 2, health = 2,
    keywords = { "deathrattle" }, on_deathrattle = function(ctx, self) ctx:gain_mana_crystals(ctx:controller(self), 1, false) end }}
return card
