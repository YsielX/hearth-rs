local card = { api_version = 1, id = "UNG_024", name = "Mana Bind",
    text = "<b>Secret:</b> When your opponent casts a spell, add a copy to your hand that costs (0).",
    set = "UNGORO", type = "spell", class = "mage", rarity = "rare", spell_school = "arcane", cost = 3,
    keywords = { "secret" }, triggers = {
        { event = "card_played", timing = "before", active_zones = { "secret" },
          condition = function(ctx, self, event)
              return event.player == ctx:opponent(ctx:controller(self)) and ctx:entity(event.entity).type == "spell"
          end,
          effect = function(ctx, self, event)
              ctx:reveal_secret(self)
              ctx:set_data(self, "waiting_copy", 1)
              ctx:give_copy(ctx:controller(self), event.entity)
          end },
        { event = "card_created", timing = "after", active_zones = { "graveyard" },
          condition = function(ctx, self, event) return event.source == self and ctx:get_data(self, "waiting_copy") == 1 end,
          effect = function(ctx, self, event)
              ctx:set_data(self, "waiting_copy", 0)
              ctx:modify(event.entity, { stat = "cost", operation = "set", value = 0 })
          end },
    } }
return card
