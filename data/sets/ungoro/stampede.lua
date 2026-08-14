local function is_beast(definition)
    for _, tag in ipairs(definition.tags or {}) do if tag == "beast" or tag == "all" then return true end end
    return false
end
local function eligible(ctx, player, definition)
    local own = ctx:player(player).class
    if definition.class == "neutral" or definition.class == own then return true end
    for _, class in ipairs(definition.classes or {}) do if class == own then return true end end
    return false
end
local card = { api_version = 1, id = "UNG_916", name = "Stampede",
    text = "Each time you play a Beast this turn, add a random Beast to your hand.",
    set = "UNGORO", type = "spell", class = "hunter", rarity = "epic", cost = 0 }
function card.on_play(ctx, self) ctx:set_data(self, "active", 1) end
card.triggers = {
    { event = "minion_played", timing = "after", active_zones = { "graveyard" },
      condition = function(ctx, self, event)
          return ctx:get_data(self, "active") == 1 and event.player == ctx:controller(self)
              and is_beast(ctx:card_definition(ctx:entity(event.entity).card_id))
      end,
      effect = function(ctx, self) ctx:continue_with("choose_beast") end },
    { event = "turn_ended", timing = "after", active_zones = { "graveyard" },
      condition = function(ctx, self, event) return event.player == ctx:controller(self) and ctx:get_data(self, "active") == 1 end,
      effect = function(ctx, self) ctx:set_data(self, "active", 0) end },
}
function card.choose_beast(ctx, self)
    local player, pool = ctx:controller(self), {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and is_beast(definition) and eligible(ctx, player, definition) then pool[#pool + 1] = card_id end
    end
    if #pool > 0 then ctx:random_value(pool, "receive_beast") end
end
function card.receive_beast(ctx, self, card_id) ctx:give_card(ctx:controller(self), card_id) end
return card
