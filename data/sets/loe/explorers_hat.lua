local card = {
    api_version = 1, id = "LOE_105", name = "Explorer's Hat",
    text = "Give a minion +1/+1 and \"<b>Deathrattle:</b> Get an Explorer's Hat.\"",
    set = "LOE", type = "spell", class = "hunter", rarity = "rare",
    cost = 1, target_mode = "required", targets = function(ctx) return ctx:minions() end,
}

function card.on_play(ctx, self, target)
    cardlib.effects.buff(ctx, target, 1, 1)
    ctx:attach_hook(target, "on_deathrattle", "LOE_105")
    cardlib.effects.grant_keyword(ctx, target, "deathrattle")
end

function card.on_deathrattle(ctx, self)
    cardlib.effects.give_card(ctx, ctx:controller(self), "LOE_105")
end

return card
