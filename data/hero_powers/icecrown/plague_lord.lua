local power = {
    api_version = 1,
    module_type = "hero_power",
    id = "ICC_832p",
    name = "Plague Lord",
    text = "<b>Choose One -</b>\n+$a3 Attack this turn;\nor Gain $d3 Armor.",
    set = "ICECROWN",
    class = "neutral",
    cost = 2,
    keywords = { "choose_one" },
}

function power.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { card_id = "ICC_832pb", label = "+3 Attack this turn" },
        { card_id = "ICC_832pa", label = "Gain 3 Armor" },
    }, "chosen")
end

function power.chosen(ctx, self, choice)
    local player = ctx:controller(self)
    if choice == "ICC_832pb" then
        cardlib.effects.buff_until_end_of_turn(ctx, ctx:player(player).hero, 3, 0)
    else
        ctx:gain_armor(player, 3)
    end
end

function power.on_choose_multiple(ctx, self)
    local player = ctx:controller(self)
    cardlib.effects.buff_until_end_of_turn(ctx, ctx:player(player).hero, 3, 0)
    ctx:gain_armor(player, 3)
end

power.tokens = {
    { id = "ICC_832pa", name = "Scarab Shell", text = "+$d3 Armor.", set = "ICECROWN", type = "spell", class = "druid", collectible = false, cost = 2 },
    { id = "ICC_832pb", name = "Spider Fangs", text = "+$a3 Attack.", set = "ICECROWN", type = "spell", class = "druid", collectible = false, cost = 2 },
}

return power
