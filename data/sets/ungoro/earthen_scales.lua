local card = { api_version = 1, id = "UNG_108", name = "Earthen Scales",
    text = "Give a friendly minion +1/+1, then gain Armor equal to its Attack.",
    set = "UNGORO", type = "spell", class = "druid", rarity = "rare", spell_school = "nature",
    cost = 2, target_mode = "required", targets = function(ctx, self) return ctx:friendly_minions(self) end }
function card.on_play(ctx, self, target)
    ctx:set_data(self, "earthen_target", target)
    cardlib.effects.buff(ctx, target, 1, 1)
    ctx:continue_with("gain_armor")
end
function card.gain_armor(ctx, self)
    local target = ctx:get_data(self, "earthen_target")
    if target ~= 0 then ctx:gain_armor(ctx:controller(self), math.max(0, ctx:entity(target).attack)) end
end
return card
