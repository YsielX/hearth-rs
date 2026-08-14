local card = {
    api_version = 1, id = "LOOT_209", name = "Dragon Soul",
    text = "After you cast 3 spells in a turn, summon a 5/5 Dragon.", set = "LOOTAPALOOZA",
    type = "weapon", class = "priest", rarity = "legendary", cost = 3, attack = 0, health = 3,
}
card.triggers = {
    { event = "turn_started", timing = "after", active_zones = { "weapon" }, effect = function(ctx, self) ctx:set_data(self, "dragon_soul_spells", 0) end },
    { event = "spell_cast", timing = "after", active_zones = { "weapon" },
      condition = function(ctx, self, event) return event.player == ctx:controller(self) and event.player_cast end,
      effect = function(ctx, self)
          local count = ctx:get_data(self, "dragon_soul_spells") + 1; ctx:set_data(self, "dragon_soul_spells", count)
          if count == 3 then ctx:summon(ctx:controller(self), "LOOT_209t") end
      end },
}
card.tokens = {{ id = "LOOT_209t", name = "Dragon Spirit", text = "", set = "LOOTAPALOOZA", type = "minion", class = "priest", collectible = false, cost = 5, attack = 5, health = 5, tags = { "undead", "dragon" } }}
return card
